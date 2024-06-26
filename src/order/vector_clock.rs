use super::LogicalClock;
use crate::order::{pairwise_max, CausalOrd, HasEvents, OrdProcess};

/// Vector Clock is used to compare if one event happens before (<) / after (>) another or if they are concurrent (None).
///
/// More precisely, events `s < t` if and only if s happens before t. This can be a result of program order (extend),
/// send-receive order on a unidirectional FIFO channel (merge) or transitivity of this relation.
///
/// # Examples
/// ```
/// use rads::order::LogicalClock;
/// use rads::order::vector_clock::VectorClock;
///
/// let e1 = VectorClock::new(0, 2);
/// assert!(e1 == e1);
/// let e2 = e1.extend();
/// assert!(e1 < e2);
/// let f1 = VectorClock::new(1, 2);
/// assert!(e1.partial_cmp(&f1) == None);
/// let f2 = f1.merge(&e1);
/// assert!(e1 < f2);
/// assert!(e2.partial_cmp(&f2) == None);
/// assert!(f1 < f2);
/// ```
#[derive(Clone)]
pub struct VectorClock {
    i: usize,
    clk: Vec<usize>,
}

impl LogicalClock for VectorClock {
    fn new(i: usize, n_procs: usize) -> Self {
        assert!(
            i < n_procs,
            "Expect 0-based index of process {i} < n_procs={n_procs}"
        );
        Self {
            i,
            clk: (0..n_procs).map(|j| usize::from(i == j)).collect(),
        }
    }
    fn extend(&self) -> Self {
        let mut e = self.clone();
        e.clk[e.i] += 1;
        e
    }
    fn merge(&self, other: &Self) -> Self {
        debug_assert_eq!(
            self.clk.len(),
            other.clk.len(),
            "Cannot merge with process that is aware of differing processes"
        );
        debug_assert!(
            self.clk[self.i] >= other.clk[self.i],
            "Process from different scheduler detected. Process' own clock's invariant broken."
        );
        Self {
            i: self.i,
            clk: pairwise_max(self.clk.iter(), other.clk.iter())
                .enumerate()
                .map(|(i, v)| v + usize::from(i == self.i))
                .collect(),
        }
    }
}

impl PartialOrd for VectorClock {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        if self.clk.len() != other.clk.len() {
            return None;
        }
        use std::cmp::Ordering::{Equal, Greater, Less};
        self.clk
            .iter()
            .zip(&other.clk)
            .try_fold(Equal, |acc, (s, t)| match (acc, s.cmp(t)) {
                (Less, Greater) | (Greater, Less) => None,
                (_, Less) | (Less, _) => Some(Less),
                (_, Greater) | (Greater, _) => Some(Greater),
                (Equal, Equal) => Some(Equal),
            })
    }
}

impl CausalOrd for VectorClock {}

impl PartialEq for VectorClock {
    fn eq(&self, other: &Self) -> bool {
        self.i == other.i && self.clk == other.clk
    }
}

pub struct VecProcess {
    i: usize,
    n_procs: usize,
    events: Vec<VectorClock>,
}

impl VecProcess {
    pub fn new(i: usize, n_procs: usize) -> Self {
        Self {
            i,
            n_procs,
            events: Vec::new(),
        }
    }
}

impl HasEvents<VectorClock> for VecProcess {
    fn last_event(&self) -> Option<&VectorClock> {
        self.events.last()
    }
    fn push_event(&mut self, e: VectorClock) {
        self.events.push(e)
    }
    fn pid(&self) -> usize {
        self.i
    }
    fn n_procs(&self) -> usize {
        self.n_procs
    }
    fn events(&self) -> &[VectorClock] {
        self.events.as_slice()
    }
}

impl OrdProcess<VectorClock> for VecProcess {}

#[cfg(test)]
mod tests {
    use crate::order::vector_clock::VecProcess;
    use crate::order::{vector_clock::VectorClock, HasEvents, LogicalClock, OrdProcess};
    use rand::Rng;

    #[test]
    fn partial_ord() {
        let e1 = VectorClock::new(0, 2);
        assert_eq!(e1.partial_cmp(&e1), Some(std::cmp::Ordering::Equal));
        let e2 = e1.extend();
        assert_eq!(e1.partial_cmp(&e2), Some(std::cmp::Ordering::Less));
        assert_eq!(e2.partial_cmp(&e1), Some(std::cmp::Ordering::Greater));
        assert_eq!(e2.partial_cmp(&e2), Some(std::cmp::Ordering::Equal));

        let f1 = VectorClock::new(1, 2);
        assert_eq!(e1.partial_cmp(&f1), None);
        assert_eq!(e2.partial_cmp(&f1), None);
        assert_eq!(f1.partial_cmp(&e1), None);
        assert_eq!(f1.partial_cmp(&e2), None);
        assert_eq!(f1.partial_cmp(&f1), Some(std::cmp::Ordering::Equal));
        let f2 = f1.merge(&e1);
        assert_eq!(e1.partial_cmp(&f2), Some(std::cmp::Ordering::Less));
        assert_eq!(e2.partial_cmp(&f2), None);
        assert_eq!(f1.partial_cmp(&f2), Some(std::cmp::Ordering::Less));
        assert_eq!(f2.partial_cmp(&e1), Some(std::cmp::Ordering::Greater));
        assert_eq!(f2.partial_cmp(&e2), None);
        assert_eq!(f2.partial_cmp(&f1), Some(std::cmp::Ordering::Greater));
        assert_eq!(f2.partial_cmp(&f2), Some(std::cmp::Ordering::Equal));
    }

    #[test]
    fn mock_scheduler() {
        let (tx3_2, rx3_2) = std::sync::mpsc::channel::<VectorClock>();
        let (tx1_2, rx1_2) = std::sync::mpsc::channel::<VectorClock>();
        let (tx3, rx3) = std::sync::mpsc::channel::<VectorClock>();

        let th1 = std::thread::spawn(move || {
            let mut p = VecProcess::new(0, 3);
            p.exec(rand_timeout);
            p.send(|e| tx1_2.send(e).unwrap());
            p.exec(rand_timeout);
            p
        });
        let th2 = std::thread::spawn(move || {
            let mut p = VecProcess::new(1, 3);
            p.exec(rand_timeout);
            p.recv(|| rx3_2.recv().unwrap());
            p.recv(|| rx1_2.recv().unwrap());
            p.send(|e| tx3.send(e).unwrap());
            p
        });
        let th3 = std::thread::spawn(move || {
            let mut p = VecProcess::new(2, 3);
            p.exec(rand_timeout);
            p.send(|e| tx3_2.send(e).unwrap());
            p.exec(rand_timeout);
            p.recv(|| rx3.recv().unwrap());
            p
        });

        let p1 = th1.join().unwrap();
        let p2 = th2.join().unwrap();
        let p3 = th3.join().unwrap();
        let p1 = p1.events();
        let p2 = p2.events();
        let p3 = p3.events();

        // Number of events
        assert_eq!(p1.len(), 3);
        assert_eq!(p2.len(), 4);
        assert_eq!(p3.len(), 4);

        // Program order --> s<t
        assert!(p1.iter().zip(&p1[1..]).all(|(s, t)| s < t));
        assert!(p2.iter().zip(&p2[1..]).all(|(s, t)| s < t));
        assert!(p3.iter().zip(&p3[1..]).all(|(s, t)| s < t));

        // Send-receive | transitive order --> s<t
        assert!(p3[..2].iter().all(|s| s < &p2[1])); // from p3 to p2
        assert!(p1[..2].iter().all(|s| s < &p2[2])); // from p1 to p2
        assert!(p3[..2].iter().all(|s| s < &p2[2]));
        assert!(p2.iter().all(|s| s < &p3[3])); // from p2 to p3
        assert!(p1[..2].iter().all(|s| s < &p3[3]));
    }

    fn rand_timeout() {
        let mut rng = rand::thread_rng();
        let t = rng.gen_range(0..=200);
        std::thread::sleep(std::time::Duration::from_millis(t));
    }
}
