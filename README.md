# Pinocchio Escrow 程序

这是一个用 Rust 与 pinocchio 框架实现的 Solana 智能合约示例，实现了一个简单的代币托管/交换（Escrow）流程。

主要功能
- Maker 创建一个 Escrow，将 Token A 存入程序控制的 vault（ATA）。
- Taker 提供 Token B，程序将 vault 中的 Token A 转给 Taker，同时将 Token B 转给 Maker。
- Maker 可以在未被接受前执行 Refund，将 Token A 退回给自身并关闭 Escrow。

代码组织
- `src/lib.rs`：程序入口与指令路由。
- `src/state.rs`：Escrow 状态定义（固定长度内存映射）。
- `src/errors.rs`：自定义错误枚举（已提供中文说明）。
- `src/instructions/`：指令实现与辅助验证工具：
  - `make.rs`：创建 Escrow 并初始化 vault
  - `take.rs`：完成交换并关闭 vault/escrow
  - `refund.rs`：取消 Escrow 并返还代币
  - `helper.rs`：账户验证/初始化工具（类似 Anchor 的约束实现）

快速构建与测试
1. 构建为 wasm：
```bash
cargo build --target wasm32-unknown-unknown
```
2. 运行测试：
```bash
cargo test
```

注意事项
- 本仓库使用 `pinocchio` 框架（无 std），部分 API 与标准 Anchor 不同。
- Escrow 状态使用固定字节布局并通过 unsafe transmute 映射，请在修改结构体字段时同时更新长度计算。

如需我把每个函数/方法都逐行注释成中文，请确认，我会继续为每个源文件补充更细粒度的注释。
