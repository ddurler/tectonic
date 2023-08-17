use std::collections::hash_map::Entry;
use std::collections::HashMap;
use std::fmt;

use crate::grid::{CellContent, Grid};
use crate::line_column::LineColumn;
use crate::neighboring_line_columns::NeighboringLineColumns;
use crate::simple_09_set::Simple09Set;

// Niveau max de récursion par défaut avec la fonction récursive `solve_try_and_see`.
// Cette fonction peut être appelée récursivement si la grille à résoudre
// est très complexe (ou si elle est en cours de construction)
// On stoppe les niveaux trop élevés de recherche par récursion qui correspondrait
// à une solution trop difficile à trouver
const DEFAULT_MAX_TRY_AND_SEE_RECURSION_LEVEL: i32 = 3;

/// Options lors de la résolution
pub enum SolvingOption {
    /// Affichage de l'action faite à chaque étape de la résolution
    StepPrintAction,

    /// Appel d'une closure avec l'action faite à chaque étape de la résolution
    StepCallbackAction(fn(&SolvingAction)),

    /// Affichage de la grille à chaque étape de la résolution
    StepPrintGrid,

    /// Appel d'une closure avec le contenu du solver à chaque étape de la résolution
    StepCallbackSolver(fn(&Solver)),

    /// Limitation du niveau de récursion lors de la recherche par 'essai' (niveau très difficile)
    /// Une valeur de 0, inhibe cette possibilité qui peut mener à des temps de calculs relativement long
    /// Une valeur d'au moins 3 est nécessaire pour des grilles très très difficiles
    MaxTryAndSeeRecursionLevel(i32),
}

impl SolvingOption {
    fn get_max_try_and_see_recursion_level(options: &[SolvingOption], default_level: i32) -> i32 {
        for option in options {
            if let SolvingOption::MaxTryAndSeeRecursionLevel(level) = option {
                return *level;
            }
        }

        default_level
    }
}

/// Action possible effectuée à chaque étape de résolution
#[derive(Debug, PartialEq, Eq)]
pub enum SolvingAction {
    /// La grille est résolue
    Solved,

    /// Initialisation des chiffres possibles pour toutes les cases
    InitPossibleNumbers,

    /// Case avec qu'une seule possibilité de chiffre
    SinglePossibleNumber(LineColumn, u8),

    /// Suppression des chiffres possibles d'une case qui sont déjà dans la zone de cette case
    NumbersInZone(LineColumn, char, Vec<u8>),

    /// Seule case possible pour un chiffre d'une zone
    OnlyNumberInZone(char, LineColumn, u8),

    /// Suppression des chiffres d'une case qui sont déjà dans une de ses cases voisines
    NumbersNeighboring(LineColumn, Vec<u8>),

    /// Suppression des chiffres d'une paire de valeurs dans les cases voisines
    DualValuesPair(LineColumn, LineColumn, LineColumn, Vec<u8>),

    // Force une valeur dans une paire de possibilité car elle mène une solution
    // après évaluation de la résolution en testant cette valeur
    TryAndSolve(LineColumn, u8, u8),

    // Suppression d'une valeur dans une paire de possibilité car elle mène à une impossibilité
    // après évaluation de la résolution en testant cette valeur
    TryAndFail(LineColumn, u8, u8),

    /// Aucune action de résolution trouvée
    NoAction,
}

impl fmt::Display for SolvingAction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Solved => {
                write!(f, "Grille résolue")
            }
            Self::InitPossibleNumbers => {
                write!(f, "Initialisation des chiffres possibles des cases...")
            }
            Self::SinglePossibleNumber(line_column, n) => {
                write!(
                    f,
                    "[{n}] est la seule possibilité pour la case {line_column}"
                )
            }
            Self::OnlyNumberInZone(c_zone, line_column, n) => {
                write!(
                    f,
                    "Zone '{c_zone}', seule la case {line_column} est possible pour [{n}]"
                )
            }
            Self::NumbersInZone(line_column, c_zone, vec_n) => {
                write!(
                    f,
                    "{vec_n:?} déjà placé dans la zone '{c_zone}' de la case {line_column}"
                )
            }
            Self::NumbersNeighboring(line_column, vec_n) => {
                write!(
                    f,
                    "{vec_n:?} est dans les cases voisines de la case {line_column}"
                )
            }
            Self::DualValuesPair(line_column_pair_1, line_column_pair_2, line_column, vec_n) => {
                write!(
                    f,
                    "{vec_n:?} impossible dans la case {line_column} selon les cases voisines {line_column_pair_1} et {line_column_pair_2}"
                )
            }
            Self::TryAndSolve(line_column, n_ok, autre_n) => {
                write!(
                    f,
                    "Entre [{n_ok}] et [{autre_n}] pour {line_column}, [{n_ok}] mène à une solution"
                )
            }
            Self::TryAndFail(line_column, n_fail, n_ok) => {
                write!(
                    f,
                    "[{n_ok}] est placé pour {line_column} car le choix de [{n_fail}] mène à une incohérence"
                )
            }
            SolvingAction::NoAction => {
                write!(f, "Aucune action de résolution trouvée")
            }
        }
    }
}

/// Cas d'erreurs possibles pendant la résolution de la grille tectonic
#[derive(Debug)]
pub enum SolvingError {
    /// Zone avec trop de cases
    ZoneTooLong(char),

    /// Case avec un chiffre plus grand que la taille de la zone
    ZoneWithUnexpectedNumber(char, LineColumn, u8),

    /// Deux cases voisines avec le même chiffre
    NeighboringWithSameNumber(LineColumn, LineColumn, u8),

    /// Deux cases d'une même zone avec le même chiffre
    ZoneWithSameNumber(char, u8),

    /// Aucun chiffre possible pour une case
    NoPossibleNumber(LineColumn),

    /// Erreur d'implémentation qui ne devrait pas arriver :)
    BadImplementation,
}

impl fmt::Display for SolvingError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SolvingError::ZoneTooLong(c_zone) => {
                write!(f, "La zone '{c_zone}' est trop grande")
            }
            SolvingError::ZoneWithUnexpectedNumber(c_zone, line_column, n) => {
                write!(
                    f,
                    "Le chiffre '{n}' en {line_column} n'est pas possible dans la zone '{c_zone}'"
                )
            }
            SolvingError::NeighboringWithSameNumber(line_column_1, line_column_2, n) => {
                write!(f, "Le chiffre {n} apparaît dans les cases voisines {line_column_1} et {line_column_2}")
            }
            Self::ZoneWithSameNumber(c_zone, n) => {
                write!(
                    f,
                    "Le chiffre '{n}' apparaît plusieurs fois dans la zone '{c_zone}'"
                )
            }
            Self::NoPossibleNumber(line_column) => {
                write!(f, "Aucun chiffre possible dans la case {line_column}")
            }
            SolvingError::BadImplementation => write!(f, "Erreur inattendue (voir source code...)"),
        }
    }
}

impl std::error::Error for SolvingError {}

/// Niveau de difficulté rencontré pendant la résolution
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub enum DifficultyLevel {
    #[default]
    Unknown,

    Easy,
    Medium,
    Hard,
    VeryHard,
}

impl fmt::Display for DifficultyLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Unknown => write!(f, "Difficulté inconnue"),
            Self::Easy => write!(f, "Difficulté facile"),
            Self::Medium => write!(f, "Difficulté moyenne"),
            Self::Hard => write!(f, "Difficile"),
            Self::VeryHard => write!(f, "Très difficile"),
        }
    }
}

/// Structure pour la résolution d'une grille tectonic
#[derive(Debug, Default)]
pub struct Solver {
    /// Grille tectonic
    grid: Grid,

    /// True lorsque toutes les cases avec un contenu `Undefined` ont été traitées
    init_cell_contents: bool,

    /// Difficulté rencontrée pendant la résolution
    pub difficulty_level: DifficultyLevel,

    /// Niveau max de récursion dans la recherche try & see
    pub max_try_and_see_recursion_level: i32,

    /// Niveau de récursion dans la rechercher try & see
    pub try_and_see_recursion_level: i32,
}

impl fmt::Display for Solver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.grid)
    }
}

impl Solver {
    /// Constructeur de l'algorithme de résolution d'après une grille
    /// Il faut utiliser en suite l'une des primitives de résolution :
    /// * `solve` : Pour rechercher une solution
    /// * `solve_step` : Pour les différentes étapes de résolution
    #[must_use]
    pub fn new(grid: &Grid) -> Self {
        Solver {
            grid: grid.clone(),
            init_cell_contents: false,
            difficulty_level: DifficultyLevel::default(),
            max_try_and_see_recursion_level: DEFAULT_MAX_TRY_AND_SEE_RECURSION_LEVEL,
            try_and_see_recursion_level: 0,
        }
    }

    /// Retourne true si la grille est résolue
    #[must_use]
    pub fn is_solved(&self) -> bool {
        for cell in self.grid.hashmap_cells.values() {
            if let CellContent::Number(_) = cell.content {
                continue;
            }
            return false;
        }
        true
    }

    /// Effectue les callbacks définis en option à chaque étape de la résolution
    fn do_step_callback(&self, options: &[SolvingOption], action: &SolvingAction) {
        for option in options {
            match option {
                SolvingOption::StepPrintAction => println!("{action}"),
                SolvingOption::StepCallbackAction(f) => f(action),
                SolvingOption::StepPrintGrid => println!("{self}"),
                SolvingOption::StepCallbackSolver(f) => f(self),
                SolvingOption::MaxTryAndSeeRecursionLevel(_) => (),
            }
        }
    }

    /// Tente de résoudre la grille en itérant continûment sur toutes les étapes de résolution
    /// Retourne true si la grille est résolue
    /// # Errors
    /// Une erreur est retournée si la grille n'est pas (ou plus) cohérente
    pub fn solve(&mut self, options: &[SolvingOption]) -> Result<bool, SolvingError> {
        // Choix optionnel pour le niveau de récursion dans les recherches très difficiles...
        self.max_try_and_see_recursion_level = SolvingOption::get_max_try_and_see_recursion_level(
            options,
            DEFAULT_MAX_TRY_AND_SEE_RECURSION_LEVEL,
        );

        #[allow(while_true)]
        while true {
            // Etape de résolution
            let action_solve_step = self.solve_step()?;

            // Callback(s) demandé(s) à chaque étape
            self.do_step_callback(options, &action_solve_step);

            // Status après cette action ?
            match action_solve_step {
                SolvingAction::Solved => return Ok(true),
                SolvingAction::NoAction => return Ok(false),
                _ => continue,
            }
        }
        Err(SolvingError::BadImplementation)
    }

    /// Applique une étape de résolution
    /// Retourne une action effectuée pour rechercher la solution
    /// Si `SolvingAction::Solved` est retourné, c'est que la grille est résolue
    /// Si `SolvingAction::NoAction` est retourné, c'est que l'algorithme a épuisé les
    /// étapes de sa recherche
    /// # Errors
    /// Une erreur est retournée si la grille n'est pas (ou plus) cohérente
    pub fn solve_step(&mut self) -> Result<SolvingAction, SolvingError> {
        // Vérifie la cohérence de la grille
        self.check()?;

        // Initialisation une fois des possibilités
        if !self.init_cell_contents {
            self.init_cell_contents = true;
            return Ok(self.solve_step_possible_numbers());
        }

        // Grille résolue ?
        if self.is_solved() {
            return Ok(SolvingAction::Solved);
        }

        // Listes des fonctions -> action / niveau de difficulté pour la résolution
        #[allow(clippy::type_complexity)]
        let vec_of_functions: Vec<(fn(&mut Self) -> SolvingAction, DifficultyLevel)> = vec![
            (Self::solve_single_possible_number, DifficultyLevel::Easy),
            (Self::solve_numbers_in_zone, DifficultyLevel::Easy),
            (Self::solve_only_number_in_zone, DifficultyLevel::Easy),
            (Self::solve_numbers_neighboring, DifficultyLevel::Medium),
            (Self::solve_dual_values_pair, DifficultyLevel::Hard),
            (Self::solve_try_and_see, DifficultyLevel::VeryHard),
        ];

        // Parcourt des fonctions de résolution à la recherche d'une action possible
        for (function, difficulty) in vec_of_functions {
            let action = function(self);
            if let SolvingAction::NoAction = action {
            } else {
                self.difficulty_level = DifficultyLevel::max(self.difficulty_level, difficulty);
                return Ok(action);
            }
        }

        // Aucune action trouvée
        Ok(SolvingAction::NoAction)
    }

    /// Etape initiale de résolution pour modifier toutes les cases avec un
    /// contenu `Undefined` en un contenu `PossibleNumbers` selon le nombre de
    /// cases dans la zone
    fn solve_step_possible_numbers(&mut self) -> SolvingAction {
        // Prépare la liste des des chiffres possibles par zone
        let mut zone_hash_map: HashMap<char, Simple09Set> = HashMap::new();
        for (c_zone, zone) in &self.grid.hashmap_zones {
            let nb_cases = zone.set_line_column.len();
            let mut simple_09_set = Simple09Set::default();
            // On ne considère que le nombre de cases de la zone pour la liste
            // des chiffres possibles dans les cases de cette zone
            for n in 1..=nb_cases {
                #[allow(clippy::cast_possible_truncation)]
                simple_09_set.insert(n as u8);
            }
            zone_hash_map.insert(*c_zone, simple_09_set);
        }
        // Recherche de toutes les cases avec un contenu 'Undefined'
        for cell in self.grid.hashmap_cells.values_mut() {
            if let CellContent::Undefined = cell.content {
                // Case à traiter, encore à Undefined...
                let simple_09_set = zone_hash_map.get(&cell.c_zone).unwrap();
                cell.content = CellContent::PossibleNumbers(*simple_09_set);
            }
        }

        // Cet étape n'est réalisée qu'une fois en début de résolution
        SolvingAction::InitPossibleNumbers
    }

    /// Etape pour identifier les cases qui n'ont qu'une seule possibilité pour le chiffre
    fn solve_single_possible_number(&mut self) -> SolvingAction {
        // Recherche de toutes les cases avec un contenu 'PossibleNumbers' avec une seule possibilité
        for cell in self.grid.hashmap_cells.values_mut() {
            if let CellContent::PossibleNumbers(simple_09_set) = cell.content.clone() {
                if simple_09_set.len() == 1 {
                    let vec_n = simple_09_set.as_vec_u8();
                    let n = vec_n[0];
                    cell.content = CellContent::Number(n);
                    return SolvingAction::SinglePossibleNumber(cell.line_column, n);
                }
            }
        }

        SolvingAction::NoAction
    }

    /// Etape pour éliminer les chiffres déjà présents dans la zone d'une case
    fn solve_numbers_in_zone(&mut self) -> SolvingAction {
        // Prépare la liste des chiffres déjà placés par zone
        let mut zone_hash_map: HashMap<char, Simple09Set> = HashMap::new();
        for (c_zone, zone) in &self.grid.hashmap_zones {
            let mut simple_09_set = Simple09Set::default();
            for line_column in &zone.set_line_column {
                let cell = self.grid.get_cell(*line_column).unwrap();
                if let CellContent::Number(n) = cell.content {
                    simple_09_set.insert(n);
                }
            }
            zone_hash_map.insert(*c_zone, simple_09_set);
        }

        // Recherche de toutes les cases avec un contenu 'PossibleNumbers'
        for cell in self.grid.hashmap_cells.values_mut() {
            if let CellContent::PossibleNumbers(cell_simple_09_set) = cell.content.clone() {
                let c_zone = cell.c_zone;
                let mut simple_09_set = *zone_hash_map.get(&c_zone).unwrap();
                simple_09_set = simple_09_set.intersection(cell_simple_09_set);
                if !simple_09_set.is_empty() {
                    // les valeurs dans simple_09_set sont déjà affectées à d'autres cases
                    // de la zone. Elles ne sont pas possibles pour cette case
                    let vec_n = simple_09_set.as_vec_u8();
                    let mut new_cell_simple_09_set = cell_simple_09_set;
                    for n in &vec_n {
                        new_cell_simple_09_set.remove(*n);
                    }
                    cell.content = CellContent::PossibleNumbers(new_cell_simple_09_set);
                    return SolvingAction::NumbersInZone(cell.line_column, c_zone, vec_n);
                }
            }
        }

        SolvingAction::NoAction
    }

    /// Etape pour identifier une seule case possible pour une valeur dans une zone
    fn solve_only_number_in_zone(&mut self) -> SolvingAction {
        // Énumération spécifique pour cette recherche
        enum OnlyNumber {
            // Une seule case identifiée pour une valeur
            OnlyLineColumn(LineColumn),

            // Plusieurs cases identifiées pour une valeur
            ManyLineColumns,
        }

        // Liste des possibilités pour toutes les zones
        let mut vec_zones_hash_map_only_numbers = Vec::new();

        // Parcourt de toutes les zones
        for (c_zone, zone) in &self.grid.hashmap_zones {
            // HashMap pour repérer les possibilités
            let mut hash_map_only_numbers = HashMap::new();

            // Parcourt des cases de la zone
            for line_column in &zone.set_line_column {
                let cell = self.grid.get_cell(*line_column).unwrap();
                if let CellContent::PossibleNumbers(simple_09_set) = cell.content {
                    // Case avec plusieurs possibilités de chiffres
                    // On renseigne le HashMap des possibilités de la zone
                    let vec_n = simple_09_set.as_vec_u8();
                    for n in vec_n {
                        match hash_map_only_numbers.entry(n) {
                            Entry::Occupied(mut e) => {
                                // Possibilité déjà vue dans la zone pour ce chiffre
                                e.insert(OnlyNumber::ManyLineColumns);
                            }
                            Entry::Vacant(e) => {
                                e.insert(OnlyNumber::OnlyLineColumn(*line_column));
                            }
                        }
                    }
                }
            }

            // Renseigne ce qu'on a trouvé pour cette zone
            vec_zones_hash_map_only_numbers.push((*c_zone, hash_map_only_numbers));
        }

        // Parcourt du vecteur de ce qu'on a trouvé pour toutes les zones
        for (c_zone, hash_map_only_numbers) in vec_zones_hash_map_only_numbers {
            // Parcourt du hash_map de la zone à la recherche de case qui serait la seule possibilité
            for (digit, only_number) in hash_map_only_numbers {
                if let OnlyNumber::OnlyLineColumn(line_column) = only_number {
                    // Il n'y a qu'une seule case possible pour ce digit dans cette zone
                    let cell = self.grid.get_mut_cell(line_column).unwrap();
                    cell.content = CellContent::Number(digit);
                    return SolvingAction::OnlyNumberInZone(c_zone, line_column, digit);
                }
            }
        }

        SolvingAction::NoAction
    }

    /// Etape pour éliminer les chiffres déjà présents dans les cases voisines
    fn solve_numbers_neighboring(&mut self) -> SolvingAction {
        // Liste des cases avec un contenu 'PossibleNumbers'
        let mut vec_line_columns_possible_numbers: Vec<(LineColumn, Simple09Set)> = Vec::new();
        for cell in self.grid.hashmap_cells.values() {
            if let CellContent::PossibleNumbers(simple_09_set) = cell.content {
                vec_line_columns_possible_numbers.push((cell.line_column, simple_09_set));
            }
        }

        // Parcourt des cases avec un contenu 'PossibleNumbers'
        for (cell_line_column, cell_simple_09_set) in vec_line_columns_possible_numbers {
            // simple_09_set des cases voisines
            let mut neighboring_simple_09_set = Simple09Set::default();
            // Parcourt des cases voisines
            let neighboring_line_columns = NeighboringLineColumns::new(
                cell_line_column,
                self.grid.min_line_column,
                self.grid.max_line_column,
            );
            for neighboring_line_column in neighboring_line_columns {
                let option_cell = self.grid.get_cell(neighboring_line_column);
                if let Some(neighboring_cell) = option_cell {
                    if let CellContent::Number(neighboring_n) = neighboring_cell.content {
                        // Simple_09_set des chiffres dans les cases voisines
                        neighboring_simple_09_set.insert(neighboring_n);
                    }
                }
            }

            let intersection_simple_09set =
                cell_simple_09_set.intersection(neighboring_simple_09_set);
            if !intersection_simple_09set.is_empty() {
                // les valeurs dans intersection_simple_09set sont déjà affectées à des cases voisines
                // Elles ne sont pas possible pour cette case en line_column
                let cell = self.grid.get_mut_cell(cell_line_column).unwrap();
                let vec_n = intersection_simple_09set.as_vec_u8();
                let mut new_cell_simple_09_set = cell_simple_09_set;
                for n in &vec_n {
                    new_cell_simple_09_set.remove(*n);
                }
                cell.content = CellContent::PossibleNumbers(new_cell_simple_09_set);
                return SolvingAction::NumbersNeighboring(cell.line_column, vec_n);
            }
        }

        SolvingAction::NoAction
    }

    /// Etape pour éliminer une paire de chiffres dans une case voisine de 2 autres
    /// cases ne pouvant avoir que ces 2 valeurs
    fn solve_dual_values_pair(&mut self) -> SolvingAction {
        // HashMap des cases avec une paire de valeurs possibles
        let mut hash_map_line_column: HashMap<LineColumn, Simple09Set> = HashMap::new();
        for (line_column, cell) in &self.grid.hashmap_cells {
            if let CellContent::PossibleNumbers(simple_09_set) = cell.content {
                if simple_09_set.len() == 2 {
                    hash_map_line_column.insert(*line_column, simple_09_set);
                }
            }
        }

        // Voir l'étude dans ./examples/Etude pairs values;
        // Le vecteur ci-dessous donne la position relative des cases à parcourir.
        // Si A est une case avec une paire de valeurs possibles, on examine toutes les cases B
        // relatives à A selon les coordonnées relatives.
        // Si ces cases A et B ont la même paire de valeurs possibles alors ces valeurs peuvent être
        // éliminées de toutes les cases C voisines dont les coordonnées relatives sont données
        let vec_pairs = [
            ((0, 1), vec![(-1, 0), (-1, 1), (1, 0), (1, 1)]),
            ((1, -1), vec![(0, -1), (1, 0)]),
            ((1, 0), vec![(0, -1), (0, 1), (1, -1), (1, 1)]),
            ((1, 1), vec![(0, 1), (1, 0)]),
        ];

        // Parcourt du hash map avec les cases une paire de valeurs possibles
        for (line_column_a, simple_09_set_a) in &hash_map_line_column {
            // Parcours du vecteur des paires de case à examiner relativement à line_column_a
            // Toutes les coordonnées de 'vec_pairs' sont relatives à 'line_column_a'
            for (relative_b, vec_relatives_c) in &vec_pairs {
                let relative_line_column_b = LineColumn::new(relative_b.0, relative_b.1);
                let line_column_b = *line_column_a + relative_line_column_b;
                // Pour savoir si line_column_b a aussi un paire de valeurs possibles, on le recherche dans hash_map_line_column
                if let Some(simple_09_set_b) = hash_map_line_column.get(&line_column_b) {
                    if simple_09_set_a == simple_09_set_b {
                        // Ici, on a identifié 2 cases a et b qui ont toutes 2 la même paire de valeurs possibles
                        // On parcourt donc les cases relatives voisines de ces 2 cases qui peuvent être
                        // affectées par ces 2 paires de valeurs qui les entourent
                        for relative_c in vec_relatives_c {
                            let relative_line_column_c =
                                LineColumn::new(relative_c.0, relative_c.1);
                            let line_column_c = *line_column_a + relative_line_column_c;
                            // Examen de la case line_column_c de la grille
                            let option_cell_c = self.grid.get_mut_cell(line_column_c);
                            if let Some(cell_c) = option_cell_c {
                                if let CellContent::PossibleNumbers(simple_09_set_c) =
                                    cell_c.content
                                {
                                    let intersection =
                                        simple_09_set_c.intersection(*simple_09_set_a);
                                    if !intersection.is_empty() {
                                        // Bingo !
                                        // On a trouve une case c avec un ensemble de valeurs possibles
                                        // qui contient une partie des paires de valeurs possibles des
                                        // cases a et b qui l'avoisinent...
                                        let vec_n = intersection.as_vec_u8();
                                        let mut new_simple_09_set_c = simple_09_set_c;
                                        for n in &vec_n {
                                            new_simple_09_set_c.remove(*n);
                                        }
                                        cell_c.content =
                                            CellContent::PossibleNumbers(new_simple_09_set_c);
                                        return SolvingAction::DualValuesPair(
                                            *line_column_a,
                                            line_column_b,
                                            line_column_c,
                                            vec_n,
                                        );
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        SolvingAction::NoAction
    }

    /// Etape pour éliminer ou forcer une valeur dans une paire de chiffres possible d'une case
    /// parce que son choix entraîne une incohérence dans la grille ou sa résolution
    fn solve_try_and_see(&mut self) -> SolvingAction {
        if self.try_and_see_recursion_level >= self.max_try_and_see_recursion_level {
            return SolvingAction::NoAction;
        }

        // HashMap des cases avec une paire de valeurs possibles
        let mut hash_map_line_column: HashMap<LineColumn, Simple09Set> = HashMap::new();
        for (line_column, cell) in &self.grid.hashmap_cells {
            if let CellContent::PossibleNumbers(simple_09_set) = cell.content {
                if simple_09_set.len() == 2 {
                    hash_map_line_column.insert(*line_column, simple_09_set);
                }
            }
        }

        // On teste brutalement la résolution en forçant les valeurs possibles pour les cases sélectionnées
        // Parcourt du hash map avec les cases une paire de valeurs possibles
        self.try_and_see_recursion_level += 1;
        for (line_column, simple_09_set) in &hash_map_line_column {
            let vec_n = simple_09_set.as_vec_u8();
            for n in &vec_n {
                // Clone la grille courante pour tenter de la résoudre en forçant la valeur de cette case
                let mut new_grid = self.grid.clone();
                let new_cell = new_grid.get_mut_cell(*line_column).unwrap();
                new_cell.content = CellContent::Number(*n);
                let mut new_solver = Solver::new(&new_grid);
                new_solver.max_try_and_see_recursion_level = self.max_try_and_see_recursion_level;
                new_solver.try_and_see_recursion_level = self.try_and_see_recursion_level;
                // println!("Recursion level = {}", self.try_and_see_recursion_level);
                match new_solver.solve(&[]) {
                    Err(_) => {
                        // Bingo !
                        // La valeur n pour line_column entraîne une incohérence de la grille
                        // On force l'autre valeur
                        let autre_n = if vec_n[0] == *n { vec_n[1] } else { vec_n[0] };
                        let cell = self.grid.get_mut_cell(*line_column).unwrap();
                        cell.content = CellContent::Number(autre_n);
                        return SolvingAction::TryAndFail(*line_column, *n, autre_n);
                    }
                    Ok(solved) => {
                        if solved {
                            // Bingo !
                            // La valeur n pour line_column permet de résoudre la grille
                            // On force cette valeur
                            let autre_n = if vec_n[0] == *n { vec_n[1] } else { vec_n[0] };
                            let cell = self.grid.get_mut_cell(*line_column).unwrap();
                            cell.content = CellContent::Number(*n);
                            return SolvingAction::TryAndSolve(*line_column, *n, autre_n);
                        }
                        // else, on n'a rien trouvé...
                    }
                }
            }
        }
        self.try_and_see_recursion_level -= 1;

        SolvingAction::NoAction
    }

    /// Vérifie la consistance de la grille
    fn check(&self) -> Result<(), SolvingError> {
        if !self.init_cell_contents {
            self.check_zone_too_long()?;
            self.check_zone_with_unexpected_number()?;
        }
        self.check_neighboring_cells()?;
        self.check_zone_numbers()?;
        self.check_cell_with_no_possible_values()?;
        Ok(())
    }

    /// Vérification (initiale) de la taille des zones
    fn check_zone_too_long(&self) -> Result<(), SolvingError> {
        // Parcourt des zones
        for (c_zone, zone) in &self.grid.hashmap_zones {
            if zone.set_line_column.len() > 9 {
                // C'est une erreur si la zone a plus de 9 cases
                return Err(SolvingError::ZoneTooLong(*c_zone));
            }
        }

        Ok(())
    }

    /// Vérification (initiale) de valeur inattendue dans une zone
    fn check_zone_with_unexpected_number(&self) -> Result<(), SolvingError> {
        // Parcourt des zones
        for (c_zone, zone) in &self.grid.hashmap_zones {
            // Parcourt des cases de la zone
            let zone_len = zone.set_line_column.len();
            for line_column in &zone.set_line_column {
                let cell = self.grid.get_cell(*line_column).unwrap();
                if let CellContent::Number(n) = cell.content {
                    // C'est une erreur si une case contient un chiffre plus grand que la taille de la zone
                    if usize::from(n) > zone_len {
                        return Err(SolvingError::ZoneWithUnexpectedNumber(
                            *c_zone,
                            *line_column,
                            n,
                        ));
                    }
                }
            }
        }

        Ok(())
    }

    /// Vérifie que pour toutes les cases avec un chiffre défini, il n'y a pas une case voisine
    /// définie avec le même chiffre
    fn check_neighboring_cells(&self) -> Result<(), SolvingError> {
        // Parcourt de toutes les cases de la grille avec un chiffre défini
        for (line_column, cell) in &self.grid.hashmap_cells {
            if let CellContent::Number(n) = cell.content {
                // Parcourt des cases voisines
                let neighboring_line_columns = NeighboringLineColumns::new(
                    *line_column,
                    self.grid.min_line_column,
                    self.grid.max_line_column,
                );
                for neighboring_line_column in neighboring_line_columns {
                    let option_cell = self.grid.get_cell(neighboring_line_column);
                    if let Some(neighboring_cell) = option_cell {
                        if let CellContent::Number(neighboring_n) = neighboring_cell.content {
                            // C'est une erreur si une case voisine contient le même chiffre
                            if n == neighboring_n {
                                return Err(SolvingError::NeighboringWithSameNumber(
                                    *line_column,
                                    neighboring_line_column,
                                    n,
                                ));
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Vérifie que pour toutes les cases définies d'une même zone, il n'y a pas un chiffre
    /// qui apparaît 2 fois
    fn check_zone_numbers(&self) -> Result<(), SolvingError> {
        // Parcourt de toutes les zones
        for (c_zone, zone) in &self.grid.hashmap_zones {
            // Init liste des chiffres définit dans la zone
            let mut zone_numbers = Simple09Set::default();
            // Parcourt des cases de la zone
            for line_column in &zone.set_line_column {
                let cell = self.grid.get_cell(*line_column).unwrap();
                if let CellContent::Number(n) = cell.content {
                    // C'est une erreur si un même chiffre apparaît plusieurs fois dans la même zone
                    if zone_numbers.contains(n) {
                        return Err(SolvingError::ZoneWithSameNumber(*c_zone, n));
                    }
                    zone_numbers.insert(n);
                }
            }
        }
        Ok(())
    }

    /// Vérifie qu'il n'y a pas une case avec aucune valeur possible
    fn check_cell_with_no_possible_values(&self) -> Result<(), SolvingError> {
        // Parcourt de toutes les cases de la grille avec une liste de valeurs possibles
        for (line_column, cell) in &self.grid.hashmap_cells {
            if let CellContent::PossibleNumbers(hash_set) = &cell.content {
                if hash_set.is_empty() {
                    return Err(SolvingError::NoPossibleNumber(*line_column));
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {

    use super::*;
    use std::fs;
    use std::path;
    use std::str::FromStr;

    #[test]
    fn test_check_ok() {
        let grid = Grid::from_str(
            "
        a1 b  b2
        b4 b  b
        c  c  c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_ok());
    }

    #[test]
    fn test_check_zone_too_long() {
        let grid = Grid::from_str(
            "
        # NOK car trop de cases dans la zone 'a'
        a1 a  a  b  a
        b  b  a  a  a
        c  c  a  a  a
        c  c  a  a  a
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_err());
    }

    #[test]
    fn test_check_zone_with_unexpected_number() {
        let grid = Grid::from_str(
            "
        # NOK car b7 n'est pas possible dans une zone de 5 cases
        a1 b  b
        b  b7  b
        c  c  c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_err());
    }

    #[test]
    fn test_check_neighboring_nok() {
        let grid = Grid::from_str(
            "
        # NOK car a1 et b1 sont voisins
        a1 b  b2
        b4 b1 b
        c  c  c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_err());
    }

    #[test]
    fn test_check_zone_nok() {
        let grid = Grid::from_str(
            "
        # NOK car 2 apparaît 2 x dans la zone b
        a1 b  b2
        b4 b2 b
        c  c  c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_err());
    }

    #[test]
    fn test_is_solved_ok() {
        let grid = Grid::from_str(
            "
        a1 b3 b2
        b4 b5 b1
        c1 c3 c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_ok());
        assert!(solver.is_solved());
    }

    #[test]
    fn test_is_solved_nok() {
        let grid = Grid::from_str(
            "
        a1 b  b2
        b4 b  b
        c  c  c2
        ",
        )
        .unwrap();

        let solver = Solver::new(&grid);

        assert!(solver.check().is_ok());
        assert!(!solver.is_solved());
    }

    #[test]
    fn test_hard_puzzle_is_solved() {
        let grid = Grid::from_str(
            "
            # Jeu Le Routard no 13 - page 38 (niveau rouge)
            a  a5 a  b  b
            c  a  a  b3 b
            d  e  e  e  f
            d5 d  e2 e  g
            d  d  g  g1 g
        ",
        )
        .unwrap();

        let mut solver = Solver::new(&grid);
        let _ = solver.solve(&[]);
        assert!(solver.is_solved());
    }

    #[test]
    fn test_all_examples() {
        // Test tous les fichiers ""./examples/*.txt" pour résolution
        for entry in fs::read_dir("./examples").unwrap() {
            let entry = entry.unwrap();
            let path = entry.path();
            let path_str = path.to_str().unwrap();
            let path_path = path::Path::new(path_str);

            if path_path.is_file()
                && path_path.file_name().unwrap().to_string_lossy().starts_with("ex")
                && path_path
                    .extension()
                    .map_or(false, |ext| ext.eq_ignore_ascii_case("txt"))
            {
                println!("Trying to solve '{path_str}'...");

                let file_content = fs::read_to_string(path_str).unwrap();
                let grid = Grid::from_str(&file_content).unwrap();
                let mut solver = Solver::new(&grid);
                let res_solver = solver.solve(&[SolvingOption::MaxTryAndSeeRecursionLevel(3)]);

                match res_solver {
                    Err(e) => println!("Erreur résolution avec le fichier '{path_str}': {e}\n"),
                    Ok(done) => assert!(done),
                }
            }
        }
    }
}
