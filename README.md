# Algorithms in Rust
My goal is to
- Implement some cool (mostly parallel) algorithms and data structures
- Learn rust
- Practice TDD

### Synchronization
Just to be clear, you should use rust's synchronization primitives. The implementation of these algorithms require
your platform's atomic support.
- [x] [Mutex Trait](src/sync/mod.rs)
- [x] [Peterson's Algorithm](src/sync/peterson.rs) for starvation-free binary mutual exclusion
- [x] [Lamport's Bakery](src/sync/lamports_bakery.rs) for starvation-free n-ary mutual exclusion (with `O(n)` time and space)

## TODO
### CS4231 Parallel & Distributed Algorithms
#### Causal Ordering
- [ ] [Happens Before Trait](src/order/mod.rs)
- [ ] Lamport's Logical Clock
- [ ] Vector Clock
- [ ] Matrix Clock
- [ ] Chandy & Lamport's Protocol (Consistent Global Snapshot) 
- [ ] Causal Order Unicast

#### Distributed Consensus
No node/link failure
- [ ] Skeen's Algorithm (Total Order Broadcast)
- [ ] Chang-Roberts Algorithm (Leader Election on Ring)
- [ ] Distributed Spanning Tree

Crash Failure, Reliable Channel, Synchronous
- [ ] F + 1 Round Protocol

No Failure, Unreliable Channel, Synchronous
- [ ] P(fail) = 1 / R Randomized Algorithm

Crash Failure, Reliable Channel, Asynchronous (FLP Impossibility Theorem)

Byzantine Failure, Reliable Channel, Synchronous
- [ ] N >= 4F + 1 Coordinator Protocol

#### Self-Stabilizing
- [ ] Self-Stabilizing Spanning Tree

### CS3223
### CS4224