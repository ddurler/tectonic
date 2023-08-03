use std::fmt;

/// Masque de bits pour les chiffres de 0 à 9
fn digit_mask_bit(digit: u8) -> u16 {
    match digit {
        0 => 1,
        1 => 2,
        2 => 4,
        3 => 8,
        4 => 16,
        5 => 32,
        6 => 64,
        7 => 128,
        8 => 256,
        9 => 512,
        _ => 0,
    }
}

/// inverse du masque de bits pour les chiffres de 0 à 9
fn not_digit_mask_bit(digit: u8) -> u16 {
    let mask = digit_mask_bit(digit);
    let mut not_mask = !mask;
    not_mask &= 1023; // Pour remettre à 0 tous les autres bits après le 9
    not_mask
}

/// Cette structure permet de gérer un set de chiffres de 0 à 9 (1 digit)
/// Comme le nombre d'éléments est limité à 10 chiffres, on utilise les
/// bits d'un u16 pour marquer les éléments du set
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Simple09Set(u16);

impl fmt::Display for Simple09Set {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{:?}", self.as_vec_u8())
    }
}
impl Simple09Set {
    /// Constructeur
    #[allow(dead_code)]
    pub fn new(digits: &[u8]) -> Self {
        let mut ret = Self::default();
        for digit in digits {
            ret.insert(*digit);
        }
        ret
    }

    /// Ajout d'un digit dans le set (sans effet si déjà présent)
    #[allow(dead_code)]
    pub fn insert(&mut self, digit: u8) {
        self.0 |= digit_mask_bit(digit);
    }

    /// Retire un digit du set (sans effet si absent)
    #[allow(dead_code)]
    pub fn remove(&mut self, digit: u8) {
        self.0 &= not_digit_mask_bit(digit);
    }

    /// Nombre de digits dans le set
    /// (Le paramètre devrait être &self mais self est optimal (16 bits au lieu d'une référence usize...))
    #[allow(dead_code)]
    pub fn len(self) -> usize {
        self.as_vec_u8().len()
    }

    /// Indique si le set est vide
    #[allow(dead_code)]
    pub fn is_empty(self) -> bool {
        self.0 == 0
    }

    /// Indique si le set contient un digit
    #[allow(dead_code)]
    pub fn contains(self, digit: u8) -> bool {
        self.0 & digit_mask_bit(digit) != 0
    }

    /// Retire du set les digits qui ne sont pas dans le set en paramètre
    #[allow(dead_code)]
    pub fn difference(mut self, other_set: Simple09Set) {
        for digit in 0..=9 {
            if self.contains(digit) && !other_set.contains(digit) {
                self.remove(digit);
            }
        }
    }

    /// Garde dans le set que les digits qui sont également dans le set en paramètre
    #[allow(dead_code)]
    pub fn intersection(&mut self, other_set: Simple09Set) {
        self.0 &= other_set.0;
    }

    /// Ajoute dans le set les digits qui sont également dans le set en paramètre
    #[allow(dead_code)]
    pub fn union(&mut self, other_set: Simple09Set) {
        self.0 |= other_set.0;
    }

    /// Retourne un Vec<u8> avec toutes les valeurs du set
    #[allow(dead_code)]
    pub fn as_vec_u8(self) -> Vec<u8> {
        let mut vec: Vec<u8> = Vec::new();
        for digit in 0..=9 {
            if self.contains(digit) {
                vec.push(digit);
            }
        }
        vec
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_simple_09_set() {
        let mut set = Simple09Set::default();

        assert!(set.is_empty());
        assert_eq!(set.len(), 0);
        for n in 0..=9 {
            assert!(!set.contains(n));
        }

        set.insert(2);
        assert!(!set.is_empty());
        assert_eq!(set.len(), 1);
        assert!(set.contains(2));

        set.insert(4);
        assert_eq!(set.len(), 2);
        assert!(set.contains(2));
        assert!(set.contains(4));

        set.remove(2);
        assert_eq!(set.len(), 1);
        assert!(!set.contains(2));
        assert!(set.contains(4));
    }

    #[test]
    fn test_simple_09_set_new() {
        let set = Simple09Set::new(&[1, 3, 7]);

        assert_eq!(set.len(), 3);
        assert!(set.contains(1));
        assert!(set.contains(3));
        assert!(set.contains(7));
    }

    #[test]
    fn test_simple_09_set_insert_remove() {
        let mut set = Simple09Set::default();

        for digit in 0..=9 {
            assert_eq!(set.len(), 0);
            set.insert(digit);
            assert_eq!(set.len(), 1);
            assert!(set.contains(digit));
            for digit2 in 0..=9 {
                if digit2 != digit {
                    assert!(!set.contains(digit2));
                }
            }
            set.remove(digit);
            assert_eq!(set.len(), 0);
            assert!(!set.contains(digit));
        }
    }

    #[test]
    fn test_simple_09_set_intersection() {
        let mut set1 = Simple09Set::new(&[1, 2, 3]);
        let set2 = Simple09Set::new(&[2, 3, 4]);

        // (1 2 3) intersection (2 3 4) -> (2 3)
        set1.intersection(set2);

        assert_eq!(set1.len(), 2);
        assert!(!set1.contains(1));
        assert!(set1.contains(2));
        assert!(set1.contains(3));
    }

    #[test]
    fn test_simple_09_set_union() {
        let mut set1 = Simple09Set::new(&[1, 2, 3]);
        let set2 = Simple09Set::new(&[2, 3, 4]);

        // (1 2 3) intersection (2 3 4) -> (2 3)
        set1.union(set2);

        assert_eq!(set1.len(), 4);
        assert!(set1.contains(1));
        assert!(set1.contains(2));
        assert!(set1.contains(3));
        assert!(set1.contains(4));
    }
}
