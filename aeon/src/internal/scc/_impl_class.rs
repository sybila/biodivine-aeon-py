use super::{Behaviour, Class};
use std::cmp::Ordering;
use std::fmt::{Display, Error, Formatter};

impl Class {
    pub fn new_empty() -> Class {
        Class(Vec::new())
    }

    /*pub fn extend(&mut self, behaviour: Behaviour) {
        self.0.push(behaviour);
        self.0.sort();
    }*/

    pub fn clone_extended(&self, behaviour: Behaviour) -> Class {
        let mut vec = self.0.clone();
        vec.push(behaviour);
        vec.sort();
        Class(vec)
    }

    /*pub fn get_vector(&self) -> Vec<Behaviour> {
        self.0.clone()
    }*/
}

impl Display for Class {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(
            f,
            "{:?}",
            self.0
                .iter()
                .map(|c| format!("{:?}", c))
                .collect::<Vec<_>>()
        )
    }
}

impl PartialOrd for Class {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// Classes actually have a special ordering - primarily, they are ordered by the
/// number of behaviours, secondarily they are ordered by the actual behaviours.
impl Ord for Class {
    fn cmp(&self, other: &Self) -> Ordering {
        if self.0.len() != other.0.len() {
            self.0.len().cmp(&other.0.len())
        } else if self.0.is_empty() {
            Ordering::Equal
        } else {
            self.0.cmp(&other.0)
        }
    }
}
