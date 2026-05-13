# Solana 按秒付费通用流支付协议

基于 Solana 区块链构建的通用化流支付协议，实现 Token 的按秒精度实时结算。适用于工资支付、订阅服务、SaaS 计费等场景。

## 核心特性

- **按秒计费** — 以秒为单位的 Token 流支付精度，支持自定义费率
- **暂停/恢复** — 付款方可随时暂停流，暂停期间不消耗 Token，恢复后自动补偿暂停时长
- **开放时长** — 支持固定时长和无限期两种流模式
- **费率调整** — 付款方可随时调整流费率
- **退款机制** — 关闭流时自动结算未消耗的 Token 退回付款方
- **多流并行** — 同一对付款方/收款方可创建多个不同 seed 的流

## 项目结构

```
stream-pay/
├── Anchor.toml                      # Anchor 配置文件
├── Cargo.toml                       # Rust workspace
├── package.json                     # TypeScript 依赖
├── tsconfig.json                    # TypeScript 配置
├── programs/stream-pay/
│   ├── Cargo.toml
│   └── src/
│       ├── lib.rs                   # #[program] 入口，6 条指令
│       ├── constants.rs             # 协议常量
│       ├── error.rs                 # 17 个自定义错误码
│       ├── events.rs                # 6 个事件结构体
│       ├── state.rs                 # Stream 账户结构体
│       ├── utils.rs                 # 按秒计费等 8 个工具函数
│       ├── tests.rs                 # 35 个 Rust 单元测试
│       ├── instructions.rs          # 模块声明
│       └── instructions/
│           ├── create_stream.rs     # 创建流
│           ├── withdraw.rs          # 提现
│           ├── pause.rs             # 暂停
│           ├── resume.rs            # 恢复
│           ├── close.rs             # 关闭流
│           └── adjust_rate.rs       # 调整费率
└── tests/
    └── stream-pay.ts               # 22 个 TypeScript 集成测试
```

## 指令说明

| 指令 | 调用方 | 说明 |
|------|--------|------|
| `create_stream` | 付款方 | 创建流支付，存入 Token 到 Vault PDA |
| `withdraw` | 收款方 | 提取已累积的 Token（按秒计算） |
| `pause_stream` | 付款方 | 暂停流，记录暂停时间点 |
| `resume_stream` | 付款方 | 恢复流，自动补偿暂停时长 |
| `close_stream` | 付款方 | 关闭流，结算未提现部分并退款 |
| `adjust_rate` | 付款方 | 调整每秒 Token 发放费率 |

## 账户结构

### Stream

| 字段 | 类型 | 说明 |
|------|------|------|
| `payer` | Pubkey | 付款方地址 |
| `recipient` | Pubkey | 收款方地址 |
| `mint` | Pubkey | SPL Token Mint |
| `vault` | Pubkey | 托管 Token 的 PDA 账户 |
| `rate` | u64 | 每秒 Token 费率（除以 PRECISION=1e9） |
| `total_amount` | u64 | 总存入金额 |
| `withdrawn_amount` | u64 | 已提取金额 |
| `start_time` | i64 | 流开始时间戳 |
| `end_time` | i64 | 流结束时间戳（0=无限期） |
| `paused_at` | i64 | 暂停时间点（0=未暂停） |
| `seed` | u64 | 用户自定义种子（同一对可开多个流） |
| `vault_bump` | u8 | Vault PDA bump |
| `bump` | u8 | Stream PDA bump |

### PDA 派生

```
stream_pda = [b"stream", payer, seed]
vault_pda  = [b"vault", stream_pda]
```

## 核心算法

### 按秒计费

```
streamed_amount = rate × elapsed_seconds / PRECISION
```

- `PRECISION = 1_000_000_000`（10^9）
- `rate = 1_000_000_000` 表示每秒 1 个 Token
- `rate = 100_000_000` 表示每秒 0.1 个 Token

### 暂停时间补偿

暂停时记录 `paused_at` 时间戳，恢复时将暂停持续时间加到 `start_time`：

```rust
pause_duration = current_time - paused_at;
stream.start_time += pause_duration;
```

### 流结算公式

```
pending_to_recipient = streamed_amount - withdrawn_amount
refund_to_payer       = total_amount - streamed_amount
```

## 快速开始

### 前置要求

- Solana CLI ≥ 1.18
- Anchor CLI ≥ 0.30
- Node.js ≥ 18

### 构建与测试

```bash
# 安装依赖
npm install

# 构建程序
anchor build

# 运行单元测试（35个）
cargo test

# 运行集成测试（需要本地验证节点）
anchor test
```

### 部署

```bash
# 本地测试网
solana-test-validator &
anchor deploy

# 主网/测试网
anchor deploy --provider.cluster devnet
```

## 测试结果

| 类别 | 数量 | 状态 |
|------|------|------|
| Rust 单元测试 | 35 | ✅ 全部通过 |
| TypeScript 集成测试 | 22 | 待运行（需 Anchor CLI 环境） |

## 协议常量

| 常量 | 值 | 说明 |
|------|-----|------|
| `PRECISION` | 1_000_000_000 | 费率精度（9位小数） |
| `BPS_DENOMINATOR` | 10_000 | 基点分母 |
| `SECONDS_PER_YEAR` | 31_536_000 | 年秒数（365天） |
| `MIN_STREAM_DURATION` | 1 秒 | 最小流时长 |
| `MAX_STREAM_DURATION` | 3_153_600_000 秒 | 最大流时长（10年） |

## 错误码

| 码 | 名称 | 说明 |
|----|------|------|
| 6000 | InvalidAmount | 金额必须大于零 |
| 6001 | InsufficientFunds | Token 账户余额不足 |
| 6003 | StreamNotStarted | 流尚未开始 |
| 6006 | Unauthorized | 非付款方或收款方调用 |
| 6007 | ArithmeticOverflow | 算术溢出 |
| 6008 | InvalidTimestamp | 时间戳无效 |
| 6009 | InvalidRate | 费率必须大于零 |
| 6010 | InvalidDuration | 时长范围无效 |
| 6011 | StreamIsPaused | 流已暂停 |
| 6012 | StreamNotPaused | 流未暂停 |
| 6014 | NothingToWithdraw | 无可提取 Token |

## 合约地址

```
Etsz3vqLMqfPToiVB1ECb2aCM1swRSaa9XLi6boH38uL
```
