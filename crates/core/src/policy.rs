//! 宿主策略 hook。
//!
//! 本模块定义 runtime 与各类 binding 之间的权限/策略接口。它不实现具体权限模型，
//! 也不决定某个 URL、路径、环境变量或进程操作是否应该被允许；这些决策应由
//! `vole-runtime` 或上层宿主通过注入 policy 完成。
//!
//! 设计目标：
//! - 让 fetch、raw net、未来 WebSocket/DNS/TCP、fs/env/process/ffi 等能力使用
//!   同一套策略结果模型。
//! - 支持默认 allow-all，保证当前 runtime 不实现权限系统时仍保持现有行为。
//! - 支持临时放行结果，方便未来实现用户交互确认、本次放行、到期放行、次数放行
//!   或测试沙箱 scoped allow。
//! - 避免在具体 binding crate 中保留私有 allow/deny list，防止权限模型碎片化。
//!
//! 当前只接入网络策略 [`NetPolicy`]，但错误模型 [`PolicyError`] 与决策模型
//! [`PolicyDecision`] 是通用的，后续可以复用于 fs/env/process/ffi 等能力。
//!
//! 使用方式：
//! - runtime 可在初始化 binding 前调用 [`init_net_policy`] 注入网络策略。
//! - binding 调用 [`check_net_url`] 进行检查。
//! - 如果没有注入策略，检查默认通过。
//! - 如果策略返回 [`PolicyDecision::AllowTemporary`]，当前操作通过；临时授权的状态
//!   持久化、过期判断和次数扣减由注入的 policy 自己维护。

use std::{fmt, sync::Arc, time::SystemTime};

use rquickjs::{Ctx, Exception, JsLifetime, Result, runtime::UserDataGuard};

/// 网络访问检查上下文。
///
/// 该结构描述一次网络能力访问请求。当前只包含目标 URL，因为 fetch 迁移阶段只需要
/// 检查 URL 级别的出站访问。后续可以在不破坏 trait 形态的前提下扩展 method、
/// headers、initiator、module specifier、redirect depth 等字段。
#[derive(Clone, Debug)]
pub struct NetAccess<'a> {
    /// 本次网络访问目标 URL。
    pub url: &'a str,
}

/// 被策略保护的宿主能力域。
///
/// 该枚举用于把通用错误和审计日志归类到具体能力，而不是为每个能力维护一套独立
/// error enum。当前只有 [`PolicyResource::Net`] 已被 fetch 使用，其它 variant 为后续
/// fs/env/process/ffi 等能力预留。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyResource {
    Net,
    Fs,
    Env,
    Process,
    Ffi,
    Shell,
    Spawn,
}

impl PolicyResource {
    /// 返回稳定的小写资源名，用于错误消息、日志和未来 CLI 输出。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Net => "net",
            Self::Fs => "fs",
            Self::Env => "env",
            Self::Process => "process",
            Self::Ffi => "ffi",
            Self::Shell => "shell",
            Self::Spawn => "spawn",
        }
    }
}

/// 策略拒绝或失败原因。
///
/// `PolicyDenyReason` 描述“为什么不允许”，[`PolicyResource`] 描述“哪个能力不允许”。
/// 二者组合后可以表达 `net explicitly denied`、`fs invalid target` 等跨能力错误。
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PolicyDenyReason {
    /// 没有授权，通常对应默认安全策略拒绝。
    NotAllowed,
    /// 被显式 deny 规则拒绝，例如 deny list 命中。
    ExplicitlyDenied,
    /// 目标非法，例如 URL/路径格式无效或无法归一化。
    InvalidTarget,
    /// 当前 runtime 或宿主策略不支持该目标/能力组合。
    Unsupported,
}

impl PolicyDenyReason {
    /// 返回稳定的小写原因文本，用于默认错误消息。
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NotAllowed => "not allowed",
            Self::ExplicitlyDenied => "explicitly denied",
            Self::InvalidTarget => "invalid target",
            Self::Unsupported => "unsupported",
        }
    }
}

/// 通用宿主策略错误。
///
/// 该类型只用于拒绝或失败结果，即 [`PolicyDecision::Deny`]。允许类结果应使用
/// [`PolicyDecision::Allow`] 或 [`PolicyDecision::AllowTemporary`] 表达。
#[derive(Clone, Debug)]
pub struct PolicyError {
    /// 被拒绝的能力域。
    pub resource: PolicyResource,
    /// 拒绝或失败原因。
    pub reason: PolicyDenyReason,
    /// 可选目标，例如 URL、文件路径、环境变量名或命令名。
    pub target: Option<String>,
    /// 可选自定义消息。存在时优先作为展示消息。
    pub message: Option<String>,
}

impl PolicyError {
    /// 构造一个通用策略错误。
    pub fn new(resource: PolicyResource, reason: PolicyDenyReason) -> Self {
        Self {
            resource,
            reason,
            target: None,
            message: None,
        }
    }

    /// 附加被拒绝的目标。
    pub fn with_target(mut self, target: impl Into<String>) -> Self {
        self.target = Some(target.into());
        self
    }

    /// 附加自定义展示消息。
    pub fn with_message(mut self, message: impl Into<String>) -> Self {
        self.message = Some(message.into());
        self
    }

    /// 生成面向 JS exception / CLI / 日志的默认消息。
    pub fn display_message(&self) -> String {
        if let Some(message) = &self.message {
            return message.clone();
        }

        let mut message = String::from("Access to ");
        message.push_str(self.resource.as_str());
        message.push(' ');
        message.push_str(self.reason.as_str());
        if let Some(target) = &self.target {
            message.push_str(": ");
            message.push_str(target);
        }
        message
    }
}

impl fmt::Display for PolicyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.display_message())
    }
}

impl std::error::Error for PolicyError {}

/// 临时放行授权。
///
/// 该类型只描述一次策略决策返回的临时授权形态，不负责保存、续期、扣减次数或审计。
/// 如果需要 `remaining_uses` 真正生效，runtime 注入的 policy 应在自身状态中完成扣减，
/// 并在下一次检查时返回新的 [`PolicyDecision`]。
#[derive(Clone, Debug, Default)]
pub struct TemporaryPolicyGrant {
    /// 临时授权的到期时间。
    pub expires_at: Option<SystemTime>,
    /// 临时授权剩余可用次数。`Some(0)` 表示已耗尽。
    pub remaining_uses: Option<u64>,
    /// 临时放行原因，例如用户交互确认、测试 sandbox scoped allow 等。
    pub reason: Option<String>,
}

impl TemporaryPolicyGrant {
    /// 构造一个没有到期时间和次数约束的临时授权描述。
    pub fn new() -> Self {
        Self::default()
    }

    /// 构造一个到指定时间过期的临时授权。
    pub fn until(expires_at: SystemTime) -> Self {
        Self {
            expires_at: Some(expires_at),
            remaining_uses: None,
            reason: None,
        }
    }

    /// 构造一个剩余指定次数的临时授权。
    pub fn for_uses(remaining_uses: u64) -> Self {
        Self {
            expires_at: None,
            remaining_uses: Some(remaining_uses),
            reason: None,
        }
    }

    /// 附加临时放行原因。
    pub fn with_reason(mut self, reason: impl Into<String>) -> Self {
        self.reason = Some(reason.into());
        self
    }

    /// 判断临时授权在给定时间点是否过期。
    pub fn is_expired(&self, now: SystemTime) -> bool {
        self.expires_at.is_some_and(|expires_at| now >= expires_at)
    }

    /// 判断临时授权次数是否已经耗尽。
    pub fn is_exhausted(&self) -> bool {
        self.remaining_uses == Some(0)
    }
}

/// 策略决策。
///
/// binding 应只关心当前操作是否可以继续：[`PolicyDecision::Allow`] 和
/// [`PolicyDecision::AllowTemporary`] 都表示当前操作允许继续；[`PolicyDecision::Deny`]
/// 表示当前操作应被拒绝。
#[derive(Clone, Debug)]
pub enum PolicyDecision {
    /// 无条件允许当前操作。
    Allow,
    /// 临时允许当前操作，并附带临时授权元数据。
    AllowTemporary(TemporaryPolicyGrant),
    /// 拒绝当前操作。
    Deny(PolicyError),
}

impl PolicyDecision {
    /// 构造无条件允许决策。
    pub fn allow() -> Self {
        Self::Allow
    }

    /// 构造临时允许决策。
    pub fn allow_temporary(grant: TemporaryPolicyGrant) -> Self {
        Self::AllowTemporary(grant)
    }

    /// 构造到指定时间过期的临时允许决策。
    pub fn allow_until(expires_at: SystemTime) -> Self {
        Self::AllowTemporary(TemporaryPolicyGrant::until(expires_at))
    }

    /// 构造剩余指定次数的临时允许决策。
    pub fn allow_for_uses(remaining_uses: u64) -> Self {
        Self::AllowTemporary(TemporaryPolicyGrant::for_uses(remaining_uses))
    }

    /// 构造拒绝决策。
    pub fn deny(error: PolicyError) -> Self {
        Self::Deny(error)
    }

    /// 当前决策是否允许操作继续。
    pub fn is_allowed(&self) -> bool {
        matches!(self, Self::Allow | Self::AllowTemporary(_))
    }

    /// 如果是拒绝决策，返回对应错误。
    pub fn deny_error(&self) -> Option<&PolicyError> {
        match self {
            Self::Deny(error) => Some(error),
            Self::Allow | Self::AllowTemporary(_) => None,
        }
    }
}

/// runtime 可选注入的网络策略。
///
/// 当前 fetch 通过该 trait 检查出站 URL。实现者可以返回允许、临时允许或拒绝。
/// 没有注入实现时，[`check_net_url`] 默认允许。
pub trait NetPolicy: Send + Sync + 'static {
    fn check_net_access(&self, access: NetAccess<'_>) -> PolicyDecision;
}

/// 存放在 rquickjs userdata 中的网络策略句柄。
#[derive(Clone)]
pub struct NetPolicyHandle {
    policy: Arc<dyn NetPolicy>,
}

unsafe impl<'js> JsLifetime<'js> for NetPolicyHandle {
    type Changed<'to> = NetPolicyHandle;
}

impl NetPolicyHandle {
    /// 创建一个可存入 JS context userdata 的策略句柄。
    pub fn new(policy: Arc<dyn NetPolicy>) -> Self {
        Self { policy }
    }

    /// 执行网络访问检查。
    pub fn check(&self, access: NetAccess<'_>) -> PolicyDecision {
        self.policy.check_net_access(access)
    }
}

/// 将网络策略注入当前 JS context。
///
/// 多次调用会保留第一次注入的策略；runtime 应在初始化 binding 前完成注入。
pub fn init_net_policy(ctx: &Ctx<'_>, policy: Arc<dyn NetPolicy>) {
    if ctx.userdata::<NetPolicyHandle>().is_none() {
        let _ = ctx.store_userdata(NetPolicyHandle::new(policy));
    }
}

/// 读取当前 JS context 中注入的网络策略。
pub fn net_policy<'a, 'js>(ctx: &'a Ctx<'js>) -> Option<UserDataGuard<'a, NetPolicyHandle>> {
    ctx.userdata::<NetPolicyHandle>()
}

/// 检查 URL 是否允许访问。
///
/// 未注入策略时默认允许。策略返回 [`PolicyDecision::AllowTemporary`] 时当前操作也会
/// 继续执行；临时授权状态由策略实现者自行维护。
pub fn check_net_url(ctx: &Ctx<'_>, url: &str) -> Result<()> {
    if let Some(policy) = net_policy(ctx) {
        match policy.check(NetAccess { url }) {
            PolicyDecision::Allow | PolicyDecision::AllowTemporary(_) => {}
            PolicyDecision::Deny(err) => {
                return Err(Exception::throw_message(ctx, &err.display_message()));
            }
        }
    }
    Ok(())
}
