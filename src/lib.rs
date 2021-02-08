pub mod sudoku;


#[cfg(test)]
mod tests {
    use std::fs;
    use super::sudoku::Sudoku;

    #[test]
    fn test_constructor() {
        let sudoku_str = String::from(".................................................................................");
        let sudoku_str2 = String::from("............2...........9........................45............3...........8.....");
        
        assert_eq!(sudoku_str, Sudoku::from_str(sudoku_str.clone()).unwrap().to_str());
        assert_eq!(sudoku_str2, Sudoku::from_str(sudoku_str2.clone()).unwrap().to_str());
    }

    #[test]
//  #[should_panic]
    fn test_too_short() {
        assert!(Sudoku::from_str(String::from("..")).is_err());
    }

    #[test]
    fn test_invalid() {
        assert!(Sudoku::from_str(String::from("123456789123456789123456789123456789123456789123456789123456789123456789")).is_err());
    }

    #[test]
    fn test_solve() {
        let sudoku_strings = fs::read_to_string("sudokus.txt").unwrap();
        let sudoku_strings_sol = fs::read_to_string("sudokus_sol.txt").unwrap();

        for (sudoku_str, sudoku_str_sol) in sudoku_strings.lines().zip(sudoku_strings_sol.lines()) {
            assert_eq!(Sudoku::from_str(sudoku_str.to_string()).unwrap().solve().unwrap().to_str(), sudoku_str_sol);
        }
    }
}
