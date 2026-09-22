# 0033. Batch profitability

> Feature 38 (Slice 7: Owner intelligence)

## Summary

This specification defines Batch profitability, an analytical model comparing tuition revenues against teacher costs, room costs, and operational overhead. Coaching center owners and administrators can inspect which batches are financially viable, adjust local cost assumptions without modifying past ledger vouchers, and identify batches operating below breakeven.

## Requirements

**User stories**:
* As an owner, I want to see which batches make money and which lose money after paying the teacher and classroom rent.
* As an administrator, I want to adjust teacher compensation rates or room cost estimates to test what if scenarios and see updated margins immediately.
* As an owner, I want a clear breakeven flag warning me if a small batch fails to cover its direct operational expenses.

**Acceptance criteria**:
* **AC-1**: Query `intelligence.get_batch_profitability` computes projected monthly tuition revenue, net collected revenue, direct costs, net margin, and margin percentage for each batch.
* **AC-2**: Command `intelligence.update_batch_cost_config` saves cost parameters (teacher monthly cost, per class rate, or percentage share of net collections, plus room and overhead allocations) in `batch_cost_configs`.
* **AC-3**: Profitability views flag any batch whose net margin is less than zero as below breakeven.
* **AC-4**: Restricts financial margin figures and cost parameters to Owner and Admin roles.

## Inline rationale

Keeping operational cost parameters in a dedicated configuration table (`batch_cost_configs`) allows owners to project costs and test what if margins dynamically without altering posted accounting entries or creating synthetic expenses. Calculating percentage shares against net collected revenue aligns with how coaching centers share tuition proceeds with senior teachers.

## Feature design

### API surface

| Endpoint | Method | Key inputs | Key outputs | Auth | Key errors |
|---|---|---|---|---|---|
| intelligence.get_batch_profitability | query | batch_id: Option<String>, session: Option<String> | Vec<BatchProfitabilityItem> | Owner, Admin | batch_not_found, unauthorized |
| intelligence.update_batch_cost_config | command | batch_id: String, teacher_monthly_cost: i64, teacher_cost_type: String, room_monthly_cost: i64, operational_overhead_cost: i64 | BatchCostConfig | Owner, Admin | batch_not_found, negative_cost |

### Value sourcing

| Action | Value produced / displayed | Source |
|---|---|---|
| get_batch_profitability | projected monthly revenue | Sum of active fee plan charge amounts for enrolled students in the batch |
| get_batch_profitability | net collected revenue | Sum of ledger payments allocated to charges belonging to the batch |
| get_batch_profitability | teacher cost | monthly_fixed uses fixed poisha; per_class multiplies by classes held; percentage_share calculates percentage of net collections |
| get_batch_profitability | room and overhead costs | Loaded from batch_cost_configs table |
| get_batch_profitability | net margin poisha | Net collected revenue minus sum of teacher, room, and overhead costs |
| get_batch_profitability | breakeven boolean | True if net margin poisha is greater than or equal to zero |

## Build tasks

1. Add core profitability calculation routine combining fee plan revenues with `batch_cost_configs`, satisfies **AC-1**, **AC-3**
2. Implement `intelligence.update_batch_cost_config` command with validation, satisfies **AC-2**
3. Create React batch profitability view with inline cost adjustment sliders and breakeven indicators, satisfies **AC-1** through **AC-4**
