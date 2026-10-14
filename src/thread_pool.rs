/*
 * Copyright 2026 Nelson Penn
 *
 * This file is part of Yet Another Commit Transformer.
 *
 * Yet Another Commit Transformer is free software: you can redistribute it
 * and/or modify it under the terms of the GNU General Public License as
 * published by the Free Software Foundation, either version 3 of the License,
 * or (at your option) any later version.
 *
 * Yet Another Commit Transformer is distributed in the hope that it will be
 * useful, but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General
 * Public License for more details.
 *
 * You should have received a copy of the GNU General Public License along with
 * Yet Another Commit Transformer. If not, see <https://www.gnu.org/licenses/>.
 */
use std::thread::JoinHandle;

pub struct ThreadPool<T: Send + Sync + 'static> {
    handles: Vec<JoinHandle<T>>,
    size: usize,
}

impl<T: Send + Sync + 'static> ThreadPool<T> {
    pub fn new(size: usize) -> Self {
        Self {
            handles: Vec::with_capacity(size),
            size,
        }
    }

    fn ensure_free_slot(&mut self) -> Option<T> {
        if self.handles.len() < self.size {
            return None;
        }
        let index = 'outer: loop {
            for (index, task) in self.handles.iter().enumerate() {
                if task.is_finished() {
                    break 'outer index;
                }
            }
        };
        let handle = self.handles.swap_remove(index);
        Some(handle.join().unwrap())
    }

    pub fn submit<F: Send + Sync + 'static + FnOnce() -> T>(&mut self, function: F) -> Option<T> {
        if self.size == 0 {
            /*
             * If size is 0, the interface can still be respected by executing
             * the function immediately in a blocking manner.
             */
            return Some(function());
        }

        let old = self.ensure_free_slot();
        self.handles.push(std::thread::spawn(function));
        old
    }

    pub fn join(self) -> Vec<T> {
        self.handles
            .into_iter()
            .map(|h| h.join().unwrap())
            .collect()
    }
}
