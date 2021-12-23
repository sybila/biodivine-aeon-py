use rayon::prelude::*;

// TODO: Something, something, FUTURES!
pub fn par_fold<T: Clone + Sync + Send, F>(items: Vec<T>, action: F) -> T
where
    F: Fn(&T, &T) -> T + Sync,
{
    if items.is_empty() {
        panic!("Empty parallel fold");
    } else if items.len() == 1 {
        return items[0].clone();
    } else {
        let data: &[T] = &items;
        let joined: Vec<T> = data
            .par_iter()
            .chunks(2)
            .map(|chunk| {
                if chunk.len() == 2 {
                    action(&chunk[0], &chunk[1])
                } else {
                    chunk[0].clone()
                }
            })
            .collect();
        return par_fold(joined, action);
    }
}
