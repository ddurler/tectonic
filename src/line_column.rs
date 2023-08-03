use std::fmt;

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

#[cfg(test)]
mod test {

    use super::*;

    #[test]
    fn test_line_column_min_max() {
        let mut lc = LineColumn::new(0, 0);
        assert_eq!(lc.line, 0);
        assert_eq!(lc.column, 0);

        lc.max(LineColumn::new(2, 3));
        assert_eq!(lc.line, 2);
        assert_eq!(lc.column, 3);

        lc.min(LineColumn::new(1, 2));
        assert_eq!(lc.line, 1);
        assert_eq!(lc.column, 2);
    }
}
