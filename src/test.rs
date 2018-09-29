use BackgroundWorker;
use std::thread;

#[cfg(test)]
pub mod tests {
    use super::*;

    #[test]
    fn initialize() {
        let mut worker: BackgroundWorker<i32, f32> = BackgroundWorker::new(|x| {x as f32});
        worker.enque(1);
        worker.join();
        assert_eq!(worker.pop().unwrap(), 1.0 as f32);
    }

    #[test]
    fn range() {
        let mut worker: BackgroundWorker<i32, f32> = BackgroundWorker::new(|x| {
            x as f32
        });
        worker.enque_vec(vec![1, 2, 3, 4, 5, 6, 7, 8]);
        worker.join();
        let mut buf = vec![0.0; 8];
        worker.pop_vec(&mut buf);
        assert_eq!(buf, vec![1., 2., 3., 4., 5., 6., 7., 8.]);
    }

    #[test]
    fn join(){
        let mut worker: BackgroundWorker<(), ()> = BackgroundWorker::new(|_| {
            thread::sleep_ms(500)
        });

        worker.enque_vec(vec![(); 10]);
        worker.join();
        let mut buf = vec![(); 10];
        worker.pop_vec(&mut buf);
        assert_eq!(buf, vec![();10]);
    }
}