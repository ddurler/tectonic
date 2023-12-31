use std::fmt;
use std::ops::Add;

/// Position (ligne, colonne) d'une case
///
/// Implicitement, la première ligne est numérotée 0 et la première colonne est également numérotée 0.
/// La première case en haut à gauche est donc en position (line: 0, column: 0).
///
/// Dans la pratique, rien n'interdit d'avoir des lignes ou des colonnes avec une numérotation négative...
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct LineColumn {
    pub line: i32,
    pub column: i32,
}

impl fmt::Display for LineColumn {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "(lin={}, col={})", self.line, self.column)
    }
}

impl LineColumn {
    #[must_use]
    pub fn new(line: i32, column: i32) -> Self {
        LineColumn { line, column }
    }

    pub fn min(&mut self, other: LineColumn) {
        self.line = i32::min(self.line, other.line);
        self.column = i32::min(self.column, other.column);
    }

    pub fn max(&mut self, other: LineColumn) {
        self.line = i32::max(self.line, other.line);
        self.column = i32::max(self.column, other.column);
    }
}

impl Add for LineColumn {
    type Output = Self;

    fn add(self, other: Self) -> Self {
        Self {
            line: self.line + other.line,
            column: self.column + other.column,
        }
    }
}

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_line_column_min_max() {
        let mut lc = LineColumn::default();
        assert_eq!(lc.line, 0);
        assert_eq!(lc.column, 0);

        lc.max(LineColumn::new(2, 3));
        assert_eq!(lc.line, 2);
        assert_eq!(lc.column, 3);

        lc.min(LineColumn::new(1, 2));
        assert_eq!(lc.line, 1);
        assert_eq!(lc.column, 2);
    }

    #[test]
    fn test_line_column_arithmetic() {
        let lc = LineColumn::new(1, 2);

        assert_eq!(lc, LineColumn::new(1, 2));
        assert_ne!(lc, LineColumn::default());

        let lc_add = lc
            + LineColumn {
                line: 2,
                column: -1,
            };
        assert_eq!(lc_add, LineColumn::new(1 + 2, 2 - 1));
    }
}
