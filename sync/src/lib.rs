mod spinlock;
pub use spinlock::SpinLock;

#[cfg(test)]
mod tests {
    use crate::SpinLock;

    #[test]
    fn spin_lock() {
        let x = SpinLock::new(Vec::new());
        std::thread::scope(|s| {
            s.spawn(|| x.lock().push(1));
            s.spawn(|| {
                let mut g = x.lock();
                g.push(2);
                g.push(3);
            });
        });
        let g = x.lock();
        assert!(g.as_slice() == [1, 2, 3] || g.as_slice() == [3, 2, 1]);
    }
}
