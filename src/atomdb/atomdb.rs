use crate::program::GroundAtom;

pub struct AtomDB {
    atoms: Vec<GroundAtom>,
}

impl AtomDB {
    pub fn new(atoms: Vec<GroundAtom>) -> Self {
        Self { atoms }
    }

    pub fn insert(&mut self, atom: &GroundAtom) {
        self.atoms.push(atom.clone());
    }

    pub fn atoms(&self) -> Vec<GroundAtom> {
        self.atoms.clone()
    }
}
