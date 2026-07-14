# RepoHop

**一键跳进正确的仓库、Agent 与会话。**

RepoHop（命令：`rhop`）是面向 AI 编程 CLI 的**本地优先**工作区路由与会话/任务管理器。在 PowerShell 中输入 `rhop`，即可选择最近项目、选择 Codex / Claude Code / OpenCode，并启动新会话（后续支持继续历史会话与独立 Git Worktree）。

RepoHop **不**实现大语言模型、不代理模型 API、不替代各 Agent CLI。它只负责项目发现、Agent 检测、会话索引、Git Worktree 管理、启动命令构造与本地历史。

## 当前状态

第二阶段最小闭环：

- `rhop doctor` — 检测本机 Agent
- `rhop scan` — 扫描配置的 `project_roots`
- 交互选择项目与 Agent → 启动**新会话**
- SQLite 启动历史

会话恢复、完整 Worktree 与安装脚本见 [docs/ROADMAP.md](docs/ROADMAP.md)。

## 环境

- Windows 11 x86_64（首要）
- PowerShell 7 或 Windows PowerShell 5.1
- 建议使用 Windows Terminal
- 至少安装 Codex CLI、Claude Code 或 OpenCode 之一

## 安装（Windows）

一键安装（PowerShell 5.1 / 7，无需管理员）：

```powershell
irm https://raw.githubusercontent.com/superdaobo/repohop/main/install.ps1 | iex
```

会将最新 Release 中的 `rhop.exe` 安装到 `%LOCALAPPDATA%\RepoHop\bin`，并写入用户 `PATH`。新开终端后执行 `rhop version`。

指定版本：

```powershell
$env:REPOPHOP_VERSION = 'v0.1.0'
irm https://raw.githubusercontent.com/superdaobo/repohop/main/install.ps1 | iex
```

卸载：

```powershell
irm https://raw.githubusercontent.com/superdaobo/repohop/main/uninstall.ps1 | iex
```

### 开发安装

```powershell
git clone https://github.com/superdaobo/repohop.git
cd repohop
cargo build --release
# 产物：target\release\rhop.exe
```

## 快速开始

**零配置。** 安装后直接：

```powershell
rhop doctor   # 查看 Agent 与自动发现的项目
rhop scan     # 从各 Agent 会话元数据刷新项目列表
rhop          # 选项目 + Agent 并启动
rhop .        # 以当前目录为项目
```

RepoHop 会**自动、只读**读取本机会话元数据来发现项目：

- Codex：`~/.codex/sessions/**/*.jsonl`
- Claude Code：`~/.claude/projects/**`
- OpenCode：`~/.local/share/opencode/opencode.db`

可选：在 `%APPDATA%\RepoHop\config.toml` 的 `project_roots` 中增加额外扫描目录。

## 命令

| 命令 | 说明 |
|------|------|
| `rhop` | 交互式跳转 |
| `rhop .` | 以当前目录为项目 |
| `rhop doctor` | 环境与 Agent 检测 |
| `rhop scan` | 更新项目缓存 |
| `rhop sessions` | 会话浏览（阶段 3） |
| `rhop worktree` | Worktree（阶段 4） |
| `rhop config` | 显示配置路径与 roots |
| `rhop version` | 版本 |

## Windows 数据路径

| 类型 | 路径 |
|------|------|
| 配置 | `%APPDATA%\RepoHop\config.toml` |
| 数据库 | `%LOCALAPPDATA%\RepoHop\repohop.db` |
| 日志 | `%LOCALAPPDATA%\RepoHop\logs` |
| Worktree | `%USERPROFILE%\.repohop\worktrees` |

## 文档

- [产品需求](docs/PRD.md)
- [架构](docs/ARCHITECTURE.md)
- [路线图](docs/ROADMAP.md)
- [会话兼容性](docs/SESSION_COMPATIBILITY.md)
- [English README](README.md)

## 许可证

MIT — 见 [LICENSE](LICENSE)。
