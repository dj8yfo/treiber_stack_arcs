use std::{fmt::Debug, sync::Arc};

use crate::cell::Cell;
use anyhow::{Result, anyhow, Context};

pub mod delete;

pub struct Cursor<T: Debug> {
    pub(super) target: Option<Arc<Cell<T>>>,
    pub(super) pre_aux: Arc<Cell<T>>,
    pub(super) pre_cell: Arc<Cell<T>>,
}

#[derive(Debug)]
pub struct NeedsUpdate;
impl std::fmt::Display for NeedsUpdate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("NeedsUpdate")
    }
}

impl<T: Debug> Cursor<T> {
    pub fn new(pre_cell: Arc<Cell<T>>, pre_aux: Arc<Cell<T>>) -> Self {
        Self {
            target: None,
            pre_cell,
            pre_aux,
        }
    }

    pub fn update(&mut self) -> Result<()>{
        match self.target {
            None => {}
            Some(ref target) => {
                if self.pre_aux.next_cmp(target) {
                    return Ok(());
                }
            }
        }

        let mut p = self.pre_aux.clone(); // expecting aux variant
        let mut n = p.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;

        drop(self.target.take());
        while !n.is_last() && !n.is_data_cell() {
            if let Err(err) = self.pre_cell.swap_in_next(p, Some(n.clone())) {
                debug_assert!({
                    println!("cursor.update {:?}", err);
                    true
                })
            }

            p = n.clone();
            n = n.next_dup().ok_or_else(|| anyhow!("unexpected None in next"))?;
        }
        self.pre_aux = p;
        self.target = Some(n);
        Ok(())
    }

    #[allow(dead_code)]
    pub fn next(&mut self) -> Result<bool> {
        let target = match self.target {
            None => return Err(anyhow!("cursor in invalid state: target is None")),
            Some(ref _target) => {
                if _target.is_last() {
                    return Ok(false);
                }
                _target
            }
        };
        self.pre_cell = target.clone();
        self.pre_aux = target
            .next_dup()
            .ok_or_else(|| anyhow!("unexpected None in next"))?;
        self.update()?;
        Ok(true)
    }

    pub fn try_insert(&self, data: T) -> Result<()> {
        let target = match self.target {
            None => return Err(anyhow!("target is none; cursor needs updating")).context(NeedsUpdate),
            Some(ref _target) => _target,
        };
        let aux = Cell::new_aux(target.clone()); // +1 target
        let data = Cell::new_data(data, aux);

        self
            .pre_aux
            .swap_in_next(target.clone(), Some(data))
            .context(NeedsUpdate)?;
        Ok(())
        
    }
}

impl<T: Debug + Copy> Cursor<T> {

    #[allow(dead_code)]
    pub fn insert(&mut self, data: T) -> Result<()> {
        while self.try_insert(data).is_err() {
            self.update()?;
        }
        Ok(())
    }

}
