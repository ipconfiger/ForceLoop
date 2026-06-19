# 需求

## 概述

使用Rust实现一个CLI程序，使用子命令来封装功能，--help可以显示子命令列表和命令的简要说明。

### 子命令列表

1. setup 在项目中初始化目录结构，状态数据，子command，Skill，hook
2. gate 门控指令，一般由钩子调用
3. status 查看当前状态
4. archive 归档开发计划

### Skills列表
1. new 创建一个新的开发目标和设计规格
2. plan 创建开发计划（多个阶段）
3. audit 审核设计规格和开发计划
4. impl 开发实现
5. review 回归验证开发结果
6. try_finish 验证是否实现开发目标

### 自定义Command列表
1. new 创建一个新的开发目标和设计规格
2. plan 创建开发计划（多个阶段）
3. audit 审核设计规格和开发计划
4. impl 开发实现
5. review 回归验证开发结果
6. try_finish 验证是否实现开发目标

## 非功能性需求
**高内聚低耦合** 子命令、Skill、自定义Command 都要抽象出trait，然后各自实现trait。（Skills和Command有关联，共用一个trait）