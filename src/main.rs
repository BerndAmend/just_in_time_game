use std::fmt;
use std::fs;
use std::str::FromStr;

#[derive(Clone, Debug, Eq, PartialEq, Hash)]
enum PieceState {
    Occupied,
    Free,
}

#[derive(Clone, Eq, PartialEq, Hash)]
struct Piece {
    id: u8,
    width: usize,
    height: usize,
    field: Vec<PieceState>,
}

impl Piece {
    fn all_variants(&self) -> Vec<Piece> {
        use std::collections::HashSet;
        let mut set = HashSet::new();

        let transform = |set: &mut HashSet<Piece>, start: Piece| {
            set.insert(start.flipped_horizontally());
            let vert = start.flipped_vertically();
            set.insert(vert.flipped_horizontally());
            set.insert(vert);
            set.insert(start);
        };

        transform(&mut set, self.clone());
        transform(&mut set, self.transposed());

        set.into_iter().collect()
    }

    fn flipped_horizontally(&self) -> Piece {
        let mut t = self.clone();

        for x in 0..self.width {
            for y in 0..self.height {
                let src_y = self.height - y - 1;
                t.field[x + y * self.width] = self.field[x + src_y * self.width].clone();
            }
        }

        t
    }

    fn flipped_vertically(&self) -> Piece {
        let mut t = self.clone();

        for x in 0..self.width {
            for y in 0..self.height {
                let src_x = self.width - x - 1;
                t.field[x + y * self.width] = self.field[src_x + y * self.width].clone();
            }
        }

        t
    }

    fn transposed(&self) -> Piece {
        let mut t = Piece {
            id: self.id,
            width: self.height,
            height: self.width,
            field: vec![PieceState::Free; self.width * self.height],
        };

        for x in 0..t.width {
            for y in 0..t.height {
                t.field[x + y * t.width] = self.field[y + x * self.width].clone();
            }
        }

        t
    }
}

impl FromStr for Piece {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<&str> = s.split_terminator('\n').collect();

        if lines.is_empty() {
            return Err("id and at least one line are required");
        }

        let width = lines.iter().fold(0, |a, b| a.max(b.len()));

        let mut result = Piece {
            id: 0,
            width,
            height: lines.len(),
            field: vec![PieceState::Free; width * lines.len()],
        };

        for line in lines.iter().enumerate() {
            for element in line.1.chars().enumerate() {
                result.field[line.0 * width + element.0] = match element.1 {
                    'X' => PieceState::Occupied,
                    ' ' => PieceState::Free,
                    _ => return Err("unexpected character"),
                }
            }
        }

        Ok(result)
    }
}

impl fmt::Display for Piece {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for e in self.field.iter().enumerate() {
            if e.0 % self.width == 0 && e.0 != 0 {
                writeln!(f)?;
            }
            write!(
                f,
                "{}",
                match *e.1 {
                    PieceState::Free => " ",
                    PieceState::Occupied => "X",
                }
            )?;
        }
        Ok(())
    }
}

#[derive(Clone, Debug)]
enum FieldState {
    Blocked,
    /// id of the element
    Occupied(u8),
    ///< score
    Free(u8),
}

#[derive(Clone, Debug)]
struct Field {
    width: usize,
    height: usize,
    field: Vec<FieldState>,
}

impl Field {
    fn place_iter<'a>(&'a self, piece: &'a Piece) -> PlaceIterator<'a> {
        PlaceIterator {
            field: self,
            piece,
            x: 0,
            y: 0,
        }
    }

    fn count(&self) -> u8 {
        self.field.iter().fold(0u8, |acc, field| {
            acc + match *field {
                FieldState::Free(score) => score,
                _ => 0,
            }
        })
    }
}

struct PlaceIterator<'a> {
    field: &'a Field,
    piece: &'a Piece,
    x: usize,
    y: usize,
}

impl<'a> Iterator for PlaceIterator<'a> {
    type Item = Field;
    fn next(&mut self) -> Option<Self::Item> {
        'main: loop {
            let field_offset_x = self.x;
            let field_offset_y = self.y;

            if field_offset_y > self.field.height - self.piece.height {
                return None; // and we're done
            }

            self.x += 1;

            if self.x > self.field.width - self.piece.width {
                self.x = 0;
                self.y += 1;
            }

            let mut ret = self.field.clone();

            for piece_x in 0..self.piece.width {
                for piece_y in 0..self.piece.height {
                    let field_x = field_offset_x + piece_x;
                    let field_y = field_offset_y + piece_y;
                    if self.piece.field[piece_x + piece_y * self.piece.width]
                        == PieceState::Occupied
                    {
                        match ret.field[field_x + field_y * ret.width].clone() {
                            FieldState::Free(_) => {
                                ret.field[field_x + field_y * ret.width] =
                                    FieldState::Occupied(self.piece.id)
                            }
                            _ => continue 'main,
                        }
                    }
                }
            }

            return Some(ret);
        }
    }
}

impl FromStr for Field {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let lines: Vec<&str> = s.split_terminator('\n').collect();
        let width = lines.iter().fold(0, |a, b| a.max(b.len()));

        let mut result = Field {
            width,
            height: lines.len(),
            field: vec![FieldState::Blocked; width * lines.len()],
        };

        for line in lines.iter().enumerate() {
            for element in line.1.chars().enumerate() {
                result.field[line.0 * width + element.0] = match element.1 {
                    ' ' => FieldState::Blocked,
                    '-' => FieldState::Free(0),
                    e @ '1'..='9' => FieldState::Free(e as u8 - b'1' + 1),
                    _ => return Err("unexpected character"),
                }
            }
        }

        Ok(result)
    }
}

impl fmt::Display for Field {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        for e in self.field.iter().enumerate() {
            if e.0 % self.width == 0 && e.0 != 0 {
                writeln!(f)?;
            }
            match *e.1 {
                FieldState::Blocked => write!(f, " "),
                FieldState::Free(0) => write!(f, "-"),
                FieldState::Free(n) => write!(f, "{}", n),
                FieldState::Occupied(n) => write!(f, "{}", (b'A' + n) as char),
            }?;
        }
        Ok(())
    }
}

struct Solution {
    start: Field,
    pieces: Vec<Vec<Piece>>,
    solutions: Vec<Field>,
}

impl Solution {
    fn new(start: &Field, pieces: &[Piece]) -> Solution {
        let pieces: Vec<Vec<Piece>> = pieces.iter().map(Piece::all_variants).collect();
        let mut solutions = vec![];
        Solution::solve(start, &pieces, &mut solutions);

        Solution {
            start: start.clone(),
            pieces,
            solutions,
        }
    }

    fn solve(state: &Field, remaining_pieces: &[Vec<Piece>], solutions: &mut Vec<Field>) {
        assert!(!remaining_pieces.is_empty());

        let top = &remaining_pieces[0];
        let rest = &remaining_pieces[1..];

        for piece in top.iter() {
            for placement in state.place_iter(&piece) {
                if rest.is_empty() {
                    solutions.push(placement);
                } else {
                    Solution::solve(&placement, &rest, solutions);
                }
            }
        }
    }

    fn highest_score(&self) -> u8 {
        self.solutions.iter().map(|f| f.count()).max().unwrap_or(0)
    }

    fn best_solutions(&self) -> Vec<Field> {
        let highest_score = self.highest_score();
        self.solutions
            .iter()
            .filter(|field| field.count() == highest_score)
            .cloned()
            .collect()
    }
}

fn main() {
    let mut args = std::env::args();

    let app_name = args.nth(0).unwrap();

    if args.len() < 2 {
        println!("usage {} <field> <pieces>", app_name);
        return;
    }

    let field_filename = args.nth(0).unwrap();
    let pieces_filename = args.nth(0).unwrap();

    println!("field={} pieces={}", field_filename, pieces_filename);

    let field: Field = fs::read_to_string(field_filename)
        .expect("couldn't read field file")
        .parse()
        .unwrap();

    let mut pieces: Vec<Piece> = vec![];
    {
        let content = fs::read_to_string(pieces_filename).expect("couldn't open pieces file");

        let mut current = vec![];
        let mut id = 0u8;
        for line in content.lines() {
            if line.is_empty() {
                if current.is_empty() {
                    panic!("Pieces file contains two empty lines");
                }
                let mut piece: Piece = current.join("\n").parse().unwrap();
                piece.id = id;
                id += 1;
                pieces.push(piece);
                current = vec![];
            } else {
                current.push(line);
            }
        }

        if !current.is_empty() {
            let mut piece: Piece = current.join("\n").parse().unwrap();
            piece.id = id;
            pieces.push(piece);
        }
    }

    let solution = Solution::new(&field, &pieces[..]);

    println!("start:\n{}\n", solution.start);

    for piece in solution.pieces.iter() {
        println!("Pieces:");
        println!("Piece {}", (b'A' + piece[0].id) as char);
        for p in piece.iter() {
            println!("{}\n", p);
        }
        println!();
    }

    println!("Possible placements:");
    for piece in solution.pieces.iter() {
        for variant in piece.iter() {
            for placement in solution.start.place_iter(&variant) {
                println!("{}\n", placement);
            }
        }
    }

    println!("Solutions:");
    for s in solution.solutions.iter() {
        println!("{}\n", s);
    }

    println!("Best solutions");
    for s in solution.best_solutions().iter() {
        println!("{}\n", s);
    }

    println!("Number of solutions {}", solution.solutions.len());
    println!("Highest score {}", solution.highest_score());
}
