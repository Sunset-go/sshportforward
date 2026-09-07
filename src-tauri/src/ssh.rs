//! 真实 SSH 隧道：基于 russh 客户端建立会话并实现本地/动态/远端端口转发。
//!
//! - `-L` 本地转发：本地监听 → `channel_open_direct_tcpip` → 双向中继
//! - `-D` 动态转发：本地 SOCKS5（仅 CONNECT，无认证）
//! - `-R` 远端转发：`tcpip_forward` 请求远端监听，服务端发起通道时按端口路由到本地目标
//!
//! 每条规则配一个 `StatCell`，经 `CountingStream` 包装 SSH 通道，
//! 写入即上传（发往 SSH 服务端）、读出即下载，按规则 id 汇总进 `TrafficMap`。
//!
//! `russh::client::Handle` 非 `Sync`，故用 `Arc<tokio::sync::Mutex<Handle>>` 在任务间共享；
//! 锁仅在打开通道/请求转发的短暂瞬间持有，数据转发走独立的 `ChannelStream`。

use std::collections::HashMap;
use std::path::PathBuf;
use std::pin::Pin;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use russh::client::{AuthResult, Handle, Handler};
use russh::keys::PrivateKeyWithHashAlg;
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::{TcpListener, TcpStream};
use tokio::sync::watch;

use crate::models::ForwardRuleOut;

type SharedHandle = Arc<tokio::sync::Mutex<Handle<ClientHandler>>>;

/// 单条规则的流量计数（字节）
#[derive(Default)]
pub struct StatCell {
    up: AtomicU64,
    down: AtomicU64,
}

impl StatCell {
    fn add_up(&self, n: u64) {
        self.up.fetch_add(n, Ordering::Relaxed);
    }
    fn add_down(&self, n: u64) {
        self.down.fetch_add(n, Ordering::Relaxed);
    }
    /// 当前 (上传, 下载) 字节数快照
    pub fn snapshot(&self) -> (u64, u64) {
        (self.up.load(Ordering::Relaxed), self.down.load(Ordering::Relaxed))
    }
}

/// 规则 id -> 流量计数
pub type TrafficMap = HashMap<String, Arc<StatCell>>;

/// `-R` 远端转发端口 -> (本地目标地址, 流量计数)
type RemoteRoutes = Arc<Mutex<HashMap<u32, (String, Arc<StatCell>)>>>;

/// 包装 SSH 通道流：poll_write 计上传，poll_read 计下载
struct CountingStream<S> {
    inner: S,
    stat: Arc<StatCell>,
}

impl<S: AsyncRead + Unpin> AsyncRead for CountingStream<S> {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let before = buf.filled().len();
        let r = Pin::new(&mut self.inner).poll_read(cx, buf);
        if let std::task::Poll::Ready(Ok(())) = &r {
            let n = (buf.filled().len() - before) as u64;
            if n > 0 {
                self.stat.add_down(n);
            }
        }
        r
    }
}

impl<S: AsyncWrite + Unpin> AsyncWrite for CountingStream<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<std::io::Result<usize>> {
        let r = Pin::new(&mut self.inner).poll_write(cx, buf);
        if let std::task::Poll::Ready(Ok(n)) = &r {
            if *n > 0 {
                self.stat.add_up(*n as u64);
            }
        }
        r
    }
    fn poll_flush(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_flush(cx)
    }
    fn poll_shutdown(
        mut self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        Pin::new(&mut self.inner).poll_shutdown(cx)
    }
}

/// 隧道建立所需的凭据（已在命令层解密）
pub struct SshConfig {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
    pub key_path: String,
    /// 私钥密码（可选，空串表示私钥未加密）
    pub passphrase: String,
}

/// 展开以 `~/` 开头的路径为绝对路径（`std::fs` 不识别 `~`）；
/// 其余路径原样返回。无 home 目录时退化为相对路径原样返回。
pub(crate) fn expand_home(path: &str) -> PathBuf {
    if let Some(rest) = path.strip_prefix("~/") {
        if let Some(home) = dirs::home_dir() {
            return home.join(rest);
        }
    }
    PathBuf::from(path)
}

#[derive(Clone)]
struct ClientHandler {
    routes: RemoteRoutes,
}

impl Handler for ClientHandler {
    type Error = russh::Error;

    async fn check_server_key(
        &mut self,
        _key: &russh::keys::PublicKeyOrCertificate,
    ) -> Result<bool, Self::Error> {
        // TODO: 主机密钥指纹固定，防中间人。当前接受所有主机密钥。
        Ok(true)
    }

    async fn server_channel_open_forwarded_tcpip(
        &mut self,
        channel: russh::Channel<russh::client::Msg>,
        _connected_address: &str,
        connected_port: u32,
        _originator_address: &str,
        _originator_port: u32,
        reply: russh::client::ChannelOpenHandle,
        _session: &mut russh::client::Session,
    ) -> Result<(), Self::Error> {
        let target = {
            let map = self.routes.lock().unwrap();
            map.get(&connected_port).cloned()
        };
        match target {
            Some((t, stat)) => {
                reply.accept().await;
                let mut stream = CountingStream { inner: channel.into_stream(), stat };
                tokio::spawn(async move {
                    if let Ok(mut local) = TcpStream::connect(&t).await {
                        let _ = tokio::io::copy_bidirectional(&mut local, &mut stream).await;
                    }
                });
            }
            None => {
                reply
                    .reject(russh::ChannelOpenFailure::AdministrativelyProhibited)
                    .await;
            }
        }
        Ok(())
    }
}

/// 一个已建立的隧道会话；drop 后关闭底层 SSH 连接
pub struct TunnelSession {
    /// 持有会话句柄以保持连接存活（drop 即断开）
    #[allow(dead_code)]
    handle: SharedHandle,
    cancel: watch::Sender<bool>,
}

impl TunnelSession {
    /// 通知所有转发任务退出并关闭会话
    pub fn shutdown(&self) {
        let _ = self.cancel.send(true);
    }
}

/// 由 tauri 管理的运行中隧道（同时只允许一个）
#[derive(Default)]
pub struct TunnelState {
    pub session: Mutex<Option<TunnelSession>>,
    pub traffic: Mutex<TrafficMap>,
}

/// 解析 `host:port`（IPv6 暂不支持）
fn parse_host_port(s: &str) -> Result<(String, u16), String> {
    let (host, port) = s
        .rsplit_once(':')
        .ok_or_else(|| format!("地址格式错误: {s}"))?;
    let port: u16 = port.parse().map_err(|_| format!("端口格式错误: {s}"))?;
    Ok((host.to_string(), port))
}

/// 建立隧道并启动各启用规则。返回会话与按规则 id 的流量计数表。
pub async fn start(
    config: SshConfig,
    rules: Vec<ForwardRuleOut>,
) -> Result<(TunnelSession, TrafficMap), String> {
    let routes: RemoteRoutes = Arc::new(Mutex::new(HashMap::new()));
    let handler = ClientHandler { routes: routes.clone() };
    let client_config = Arc::new(russh::client::Config::default());

    let mut handle = russh::client::connect(
        client_config,
        (config.host.clone(), config.port),
        handler,
    )
    .await
    .map_err(|e| format!("连接 {}:{} 失败: {e}", config.host, config.port))?;

    let auth = if !config.key_path.is_empty() {
        let key_path = expand_home(&config.key_path);
        let passphrase = if config.passphrase.is_empty() {
            None
        } else {
            Some(config.passphrase.as_str())
        };
        let key = russh::keys::load_secret_key(&key_path, passphrase)
            .map_err(|e| format!("加载私钥失败: {e}"))?;
        let key = PrivateKeyWithHashAlg::new(Arc::new(key), None);
        handle
            .authenticate_publickey(config.username.clone(), key)
            .await
    } else {
        handle
            .authenticate_password(config.username.clone(), config.password.clone())
            .await
    };

    match auth {
        Ok(AuthResult::Success) => {}
        Ok(AuthResult::Failure { .. }) => return Err("认证失败：用户名或密码/密钥错误".to_string()),
        Err(e) => return Err(format!("认证失败: {e}")),
    }

    let handle: SharedHandle = Arc::new(tokio::sync::Mutex::new(handle));
    let (cancel, cancel_rx) = watch::channel(false);

    let mut traffic: TrafficMap = HashMap::new();
    for rule in rules.into_iter().filter(|r| r.enabled) {
        let stat = Arc::new(StatCell::default());
        let res = match rule.rule_type.as_str() {
            "local" => spawn_local(handle.clone(), &rule, stat.clone(), cancel_rx.clone()).await,
            "dynamic" => spawn_dynamic(handle.clone(), &rule, stat.clone(), cancel_rx.clone()).await,
            "remote" => spawn_remote(handle.clone(), &rule, &routes, stat.clone(), cancel_rx.clone()).await,
            other => Err(format!("未知转发类型: {other}")),
        };
        if let Err(e) = res {
            let _ = cancel.send(true);
            return Err(e);
        }
        traffic.insert(rule.id.clone(), stat);
    }

    Ok((TunnelSession { handle, cancel }, traffic))
}

/// `-L` 本地转发
async fn spawn_local(
    handle: SharedHandle,
    rule: &ForwardRuleOut,
    stat: Arc<StatCell>,
    mut cancel: watch::Receiver<bool>,
) -> Result<(), String> {
    let (target_host, target_port) = parse_host_port(&rule.target_addr)?;
    let listener = TcpListener::bind(&rule.listen_addr)
        .await
        .map_err(|e| format!("监听 {} 失败: {e}", rule.listen_addr))?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel.changed() => break,
                accepted = listener.accept() => {
                    match accepted {
                        Ok((mut local, _)) => {
                            let h = handle.clone();
                            let host = target_host.clone();
                            let stat = stat.clone();
                            tokio::spawn(async move {
                                let channel = h
                                    .lock()
                                    .await
                                    .channel_open_direct_tcpip(host.as_str(), target_port as u32, "127.0.0.1", 0)
                                    .await;
                                if let Ok(channel) = channel {
                                    let mut stream = CountingStream { inner: channel.into_stream(), stat };
                                    let _ = tokio::io::copy_bidirectional(&mut local, &mut stream).await;
                                }
                            });
                        }
                        Err(_) => break,
                    }
                }
            }
        }
    });
    Ok(())
}

/// `-D` 动态转发：本地 SOCKS5（仅 CONNECT，无认证）
async fn spawn_dynamic(
    handle: SharedHandle,
    rule: &ForwardRuleOut,
    stat: Arc<StatCell>,
    mut cancel: watch::Receiver<bool>,
) -> Result<(), String> {
    let listener = TcpListener::bind(&rule.listen_addr)
        .await
        .map_err(|e| format!("监听 {} 失败: {e}", rule.listen_addr))?;

    tokio::spawn(async move {
        loop {
            tokio::select! {
                _ = cancel.changed() => break,
                accepted = listener.accept() => {
                    match accepted {
                        Ok((stream, _)) => {
                            let h = handle.clone();
                            let stat = stat.clone();
                            tokio::spawn(async move {
                                let _ = relay_socks5(h, stream, stat).await;
                            });
                        }
                        Err(_) => break,
                    }
                }
            }
        }
    });
    Ok(())
}

/// 最小 SOCKS5 CONNECT 中继：协商 → 解析目标 → direct-tcpip → 双向转发
async fn relay_socks5(
    handle: SharedHandle,
    mut client: TcpStream,
    stat: Arc<StatCell>,
) -> Result<(), ()> {
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    let mut buf = [0u8; 2];
    client.read_exact(&mut buf).await.map_err(|_| ())?;
    if buf[0] != 0x05 {
        return Err(());
    }
    let nmethods = buf[1] as usize;
    let mut methods = vec![0u8; nmethods];
    client.read_exact(&mut methods).await.map_err(|_| ())?;
    client.write_all(&[0x05, 0x00]).await.map_err(|_| ())?;

    let mut head = [0u8; 4];
    client.read_exact(&mut head).await.map_err(|_| ())?;
    if head[0] != 0x05 || head[1] != 0x01 {
        // 仅支持 CONNECT
        client
            .write_all(&[0x05, 0x07, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
            .await
            .map_err(|_| ())?;
        return Err(());
    }
    let host = match head[3] {
        0x01 => {
            let mut a = [0u8; 4];
            client.read_exact(&mut a).await.map_err(|_| ())?;
            format!("{}.{}.{}.{}", a[0], a[1], a[2], a[3])
        }
        0x03 => {
            let mut len = [0u8; 1];
            client.read_exact(&mut len).await.map_err(|_| ())?;
            let mut d = vec![0u8; len[0] as usize];
            client.read_exact(&mut d).await.map_err(|_| ())?;
            String::from_utf8_lossy(&d).to_string()
        }
        0x04 => {
            let mut a = [0u8; 16];
            client.read_exact(&mut a).await.map_err(|_| ())?;
            let mut parts = Vec::with_capacity(8);
            for i in 0..8 {
                parts.push(format!("{:x}", u16::from_be_bytes([a[i * 2], a[i * 2 + 1]])));
            }
            parts.join(":")
        }
        _ => return Err(()),
    };
    let mut port = [0u8; 2];
    client.read_exact(&mut port).await.map_err(|_| ())?;
    let port = u16::from_be_bytes(port);

    let channel = handle
        .lock()
        .await
        .channel_open_direct_tcpip(host.as_str(), port as u32, "127.0.0.1", 0)
        .await;

    match channel {
        Ok(channel) => {
            client
                .write_all(&[0x05, 0x00, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                .await
                .map_err(|_| ())?;
            let mut stream = CountingStream { inner: channel.into_stream(), stat };
            let _ = tokio::io::copy_bidirectional(&mut client, &mut stream).await;
            Ok(())
        }
        Err(_) => {
            client
                .write_all(&[0x05, 0x05, 0x00, 0x01, 0, 0, 0, 0, 0, 0])
                .await
                .map_err(|_| ())?;
            Err(())
        }
    }
}

/// `-R` 远端转发
async fn spawn_remote(
    handle: SharedHandle,
    rule: &ForwardRuleOut,
    routes: &RemoteRoutes,
    stat: Arc<StatCell>,
    mut cancel: watch::Receiver<bool>,
) -> Result<(), String> {
    let (bind_host, bind_port) = parse_host_port(&rule.listen_addr)?;
    let actual_port = handle
        .lock()
        .await
        .tcpip_forward(bind_host.clone(), bind_port as u32)
        .await
        .map_err(|e| format!("远端转发 {} 失败: {e}", rule.listen_addr))?;
    routes
        .lock()
        .unwrap()
        .insert(actual_port, (rule.target_addr.clone(), stat));

    tokio::spawn(async move {
        let _ = cancel.changed().await;
        let _ = handle
            .lock()
            .await
            .cancel_tcpip_forward(bind_host, actual_port)
            .await;
    });
    Ok(())
}
