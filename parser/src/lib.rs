use data::{Token, TokenType, Literal, Watcher};
use std::fmt;

/// Keeps track of time signature and smallest visible beat for a staff.
struct Time {
    beats_per_measure: u32,
    dominant_beat: u32,
    fidelity: u32,
    current_beat: u32,
    total_beats_counted: u32,
}

impl Time {
    /// Creates a new `Time` struct with default settings:
    /// 
    /// `beats_per_measure = 4, dominant_beat = 4, fidelity = 16, current_beat = 0, total_beats_counted = 0`
    fn new() -> Time {
        Time {
            beats_per_measure: 4,
            dominant_beat: 4,
            fidelity: 16,
            current_beat: 0,
            total_beats_counted: 0,
        }
    }

    /// Sets the time signature.
    pub fn set_signature(&mut self, beats_per_measure: u32, dominant_beat: u32) {
        // beats per measure and dominant beat cannot be less than or equal to 0
        self.beats_per_measure = if beats_per_measure > 0 { beats_per_measure } else { 1 };
        self.dominant_beat = if dominant_beat > 0 { dominant_beat } else { 1 };
    }

    /// Gets the time signature as a tuple.
    pub fn get_signature(&self) -> (u32, u32) {
        (self.beats_per_measure, self.dominant_beat)
    }

    /// Sets the beat fidelity (or resolution; granularity).
    pub fn set_fidelity(&mut self, fidelity: u32) {
        // fidelity cannot be less than or equal to 0
        self.fidelity = if fidelity > 0 { fidelity } else { 1 };
    }

    /// Gets the beat fidelity.
    pub fn get_fidelity(&self) -> u32 {
        self.fidelity
    }

    /// Gets the current beat as the beat number, 'e', '&', or 'a'.
    pub fn get_beat(&self) -> String {
        self.get_beat_at(self.current_beat)
    }

    /// Increments the current beat to the next beat.
    pub fn increment_beat(&mut self) {
        self.current_beat = (self.current_beat + 1) % self.total_beats_per_measure();
        self.total_beats_counted += 1;
    }

    /// Returns the total number of possible beats and fractional beats within a given measure.
    fn total_beats_per_measure(&self) -> u32 {
        self.beats_per_measure * (self.fidelity / self.dominant_beat)
    }

    /// Gets the beat at the provided beat position within a measure.
    /// Returned result will either be the beat number, 'e', '&', or 'a'.
    fn get_beat_at(&self, pos: u32) -> String {
        let beat_resolution = self.fidelity as f32 / self.dominant_beat as f32;
        let beat_div = pos % beat_resolution as u32;
        let current_beat = pos / beat_resolution as u32;

        if beat_div == 0 { (current_beat + 1).to_string() }
        else if beat_div as f32 / beat_resolution == 0.25 { String::from('e') }
        else if beat_div as f32 / beat_resolution == 0.5 { String::from('&') }
        else if beat_div as f32 / beat_resolution == 0.75 { String::from('a') }
        else { String::from('.') }
    }
}

impl fmt::Display for Time {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        // notes have 3 starting spaces "Nm_" where 'N' is the note name, 'm' is the modifier, and '_' is
        // a blank space; set beats to initially be 3 blank spaces
        let mut beats = String::from("   ");
        for b in 0..self.total_beats_counted {
            let beat = self.get_beat_at(b % self.total_beats_per_measure());
            // add a space for non-beat counted chars like bar-line characters
            if beat == "1" { beats.push_str(" "); }
            // beats that are 1 char in length will be represented as "_n_" while 2 length beats are "_nn"
            // where 'n' is a number and '_' is a space
            beats.push_str(&format!(
                " {}{}",
                beat,
                if beat.len() == 1 { " " } else { "" }
            ));
        }
        write!(f, "{}", beats)
    }
}

/// Contains all of the tablature numbers and note names and manages formatting the printed results.
struct Staff {
    notes: Vec<String>,
    tabs: Vec<String>,
    time: Time,
    has_tabs: bool,
    string_pos: usize,
}

impl Staff {
    /// Creates a new staff for adding notes and tabs to.
    pub fn new() -> Staff {
        Staff {
            notes: vec![],
            tabs: vec![],
            time: Time::new(),
            has_tabs: false,
            string_pos: 0,
        }
    }

    /// Sets the time signature of the staff.
    /// 
    /// # Errors
    /// 
    /// This function errors if tabs have already been added.
    pub fn set_time_signature(&mut self, (beats_per_measure, dominant_beat): (u32, u32)) -> Result<(), String> {
        if !self.has_tabs {
            self.time.set_signature(beats_per_measure, dominant_beat);
            Ok(())
        } else {
            Err(String::from("[IE_pr-st-fn(SIG)]: cannot set time signature after tabs have been added.\n"))
        }
    }

    /// Sets the beat fidelity of the staff.
    /// 
    /// # Errors
    /// 
    /// This function errors if tabs have already been added.
    pub fn set_time_fidelity(&mut self, fidelity: u32) -> Result<(), String> {
        if !self.has_tabs {
            self.time.set_fidelity(fidelity);
            Ok(())
        } else {
            Err(String::from("[IE_pr-st-fn(FID)]: cannot set fidelity after tabs have been added.\n"))
        }
    }

    /// Adds a note to the staff.
    /// 
    /// # Errors
    /// 
    /// This function errors if tabs have already been added.
    pub fn add_note(&mut self, note: String) -> Result<(), String> {
        if !self.has_tabs {
            self.notes.push(note);
            self.tabs.push(String::new());
            self.string_pos = self.notes.len() - 1;
            Ok(())
        } else {
            Err(String::from("[IE_pr-st-fn(ADN)]: cannot add note after tabs have been added.\n"))
        }
    }

    /// Adds a guitar tab to the staff.
    pub fn add_tab(&mut self, tab: &String) {
        // checks the current beat; if current beat is a downbeat, add a bar-line character
        self.check_beat();

        // make sure the tabs vector has a string available at the string position
        if let Some(tab_lane) = self.tabs.get_mut(self.string_pos) {
            // format the tab so that single char tabs are formatted "-n-" while two char tabs are "-nn"
            tab_lane.push_str(&format!(
                "-{}{}",
                tab,
                if tab.len() == 1 { "-" } else { "" }
            ));
            self.has_tabs = true;
            self.update_string_pos();
        }
    }

    /// Adds an empty tab to the staff.
    pub fn add_empty(&mut self) {
        // checks the current beat; if current beat is a downbeat, add a bar-line character
        self.check_beat();

        // make sure the tabs vector has a string available at the string position
        // format empty tabs as "---"; all tabs will be 3 chars in length
        if let Some(tab_lane) = self.tabs.get_mut(self.string_pos) {
            tab_lane.push_str("---");
            self.has_tabs = true;
            self.update_string_pos();
        }
    }

    /// Adds empty tabs to the staff until the string position resets back to its starting position.
    pub fn add_next(&mut self) {
        // loop through from the current string position to the first (and final) string position
        for pos in (0..=self.string_pos).rev() {
            // checks the current beat; if current beat is a downbeat, add a bar-line character
            self.check_beat();

            // make sure the tabs vector has a string available at the string position
            // format empty tabs as "---"; all tabs will be 3 chars in length
            if let Some(tab_lane) = self.tabs.get_mut(pos) {
                tab_lane.push_str("---");
                self.has_tabs = true;
            }
            self.update_string_pos();
        }
    }

    /// Adds empty tabs for the provided amount.
    pub fn add_spread_empty(&mut self, amt: u32) {
        for _ in 0..amt {
            self.add_empty();
        }
    }

    /// Adds empty tabs for the provided amount, each time adding empty tabs until the string position
    /// resets back to its starting position.
    pub fn add_spread_next(&mut self, amt: u32) {
        for _ in 0..amt {
            self.add_next();
        }
    }

    /// Updates the current string position. String position starts at `note.len() - 1` and decrements
    /// until `0` then resets.
    fn update_string_pos(&mut self) {
        self.string_pos = if self.string_pos == 0 {
            self.time.increment_beat();
            self.notes.len() - 1
        } else {
            self.string_pos - 1
        };
    }

    /// Checks if the current beat is a downbeat and add a bar-line character if so.
    fn check_beat(&mut self) {
        if self.time.get_beat() == "1" {
            if let Some(tab_lane) = self.tabs.get_mut(self.string_pos) {
                tab_lane.push_str("|");
            }
        }
    }
}

impl fmt::Display for Staff {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut tabs = String::new();
        // zip together both notes and tabs to print to their respective lines
        for (n, t) in self.notes.iter().rev().zip(self.tabs.iter()) {
            tabs.push_str(&format!(
                "{} {}\n",
                if n.len() == 1 { format!("{} ", n) } else { n.to_string() },
                t
            ));
        }
        write!(f, "{}\n{}\n", tabs, self.time)
    }
}

/// Parses and contains options provided from the source token input and outputs them in a
/// friendly format.
struct StaffOptions {
    time: Time,
}

impl StaffOptions {
    /// Creates a new `StaffOptions` struct with default properties.
    pub fn new() -> StaffOptions {
        StaffOptions {
            time: Time::new(),
        }
    }

    /// Parses provided options literal into formatted option data types.
    /// 
    /// # Errors
    /// 
    /// This function errors if the provided literal is not an options literal, the options have syntax
    /// errors, or if the option name or value is not valid.
    pub fn set(&mut self, options: &str) -> Result<(), String> {
        // used to log all errors that occur
        let mut errors = String::new();

        // each option will be separated by a semicolon
        for op in options.split(';') {
            // if an error occurs, log it and continue the loop
            if let Err(e) = self.parse_option(op) {
                errors.push_str(&e);
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Gets the time signature.
    pub fn get_time_signature(&self) -> (u32, u32) {
        self.time.get_signature()
    }

    /// Gets the beat fidelity.
    pub fn get_time_fidelity(&self) -> u32 {
        self.time.get_fidelity()
    }

    /// Parses provided option reference string into a formatted option data type.
    /// 
    /// # Errors
    /// 
    /// This function errors if the provided option is not set or the option does not exist.
    fn parse_option(&mut self, option: &str) -> Result<(), String> {
        // options will be structured as "option=value" and will be split based on that format
        let o: Vec<&str> = option.trim().split('=').collect();

        // check to make sure there are 2 values in the vector; if not, then return an error
        if o.len() < 2 {
            return Err(format!("\tOption \"{:?}\" has not been set to a value.\n", o))
        }

        // match based on the option name and the use the value for processing
        match (o[0].trim(), o[1].trim()) {
            // a time signature option will have the format "n/n" where 'n' is a number
            // this will be further split at the '/' character to get the beats per measure
            // and dominant beat values
            ("time", time_sig) => self.parse_time_signature(time_sig),
            // the fidelity value will be a single number value
            ("fidelity", fidelity) => self.parse_fidelity(fidelity),
            // any other option provided is an error
            (unknown_option, _) => Err(format!("\tOption \"{}\" does not exist.\n", unknown_option)),
        }
    }

    /// Parse provided reference string into a time signature.
    /// 
    /// # Errors
    /// 
    /// This function errors if the provided reference string is improperly formatted or the values
    /// on either side of the '/' cannot be parsed into whole integers.
    fn parse_time_signature(&mut self, time_signature: &str) -> Result<(), String> {
        let t: Vec<&str> = time_signature.trim().split('/').collect();
        if t.len() < 2 {
            return Err(format!("\tTime signature option \"{}\" is improperly formatted. Format should equal \"n/n\" where 'n' is a whole integer.\n", time_signature))
        }

        match (t[0].trim().parse::<u32>(), t[1].trim().parse::<u32>()) {
            (Ok(b), Ok(d)) => {
                self.time.set_signature(b, d);
                Ok(())
            },
            (Err(e_b), Err(e_d)) => {
                Err(format!("\tCould not parse time signature \"{:?}\" into numbers: {:?}\n", (t[0], t[1]), (e_b, e_d)))
            },
            (Err(e_b), _) => {
                Err(format!("\tCould not parse beats per measure (numerator) \"{}\" into a number: {}\n", t[0], e_b))
            },
            (_, Err(e_d)) => {
                Err(format!("\tCould not parse dominant beat (denominator) \"{}\" into a number: {}\n", t[1], e_d))
            },
        }
    }

    /// Parse the provided reference string into a beat fidelity (or resolution; granularity) whole integer.
    /// 
    /// # Errors
    /// 
    /// This function errors if the provided reference string is cannot be parsed into a number.
    fn parse_fidelity(&mut self, fidelity: &str) -> Result<(), String> {
        match fidelity.trim().parse::<u32>() {
            Ok(f) => {
                self.time.set_fidelity(f);
                Ok(())
            },
            Err(e) => Err(format!("\tCould not parse beat fidelity \"{}\" into a number: {}\n", fidelity, e)),
        }
    }
}

/// Manages a list of `Staff` structs by adding new staffs as needed and setting global options on them.
struct StaffManager {
    staffs: Vec<Staff>,
    options: StaffOptions,
}

impl StaffManager {
    /// Creates a new `StaffManager` with an empty list of staffs.
    pub fn new() -> StaffManager {
        StaffManager {
            staffs: vec![],
            options: StaffOptions::new(),
        }
    }

    /// Adds a note to the most recently added staff. If the staff list is empty, or the most recent staff
    /// already has tabs (and therefore adding a new note would break it), then a new staff is created
    /// with the provided note inserted into it.
    /// 
    /// # Errors
    /// 
    /// This function errors if a note insertion is attempted on a staff that has tabs.
    pub fn add_note(&mut self, note: String) {
        // these are the only possible values that can exist when checking the staff list:
        // staff exists: if staff has tabs, create new staff; else, continue
        // staff does not exist: create new staff
        match self.staffs.last() {
            Some(staff) if staff.has_tabs => self.create_staff(),
            None => self.create_staff(),
            _ => (),
        }

        // staff will either be a new staff or a staff with no tabs; safe to unwrap value
        if let Some(staff) = self.staffs.last_mut() {
            staff.add_note(note).unwrap();
        }
    }

    /// Adds a tab to the most recently added staff.
    pub fn add_tab(&mut self, tab: &String) {
        if let Some(staff) = self.staffs.last_mut() {
            staff.add_tab(tab);
        }
    }

    /// Adds an empty tab to the most recently added staff.
    pub fn add_empty(&mut self) {
        if let Some(staff) = self.staffs.last_mut() {
            staff.add_empty();
        }
    }

    /// Adds empty tabs to the most recently added staff until the guitar string position resets.
    pub fn add_next(&mut self) {
        if let Some(staff) = self.staffs.last_mut() {
            staff.add_next();
        }
    }

    /// Adds empty tabs to the most recently added staff for the provided amount of times.
    pub fn add_spread_empty(&mut self, amt: u32) {
        if let Some(staff) = self.staffs.last_mut() {
            staff.add_spread_empty(amt);
        }
    }

    /// Adds empty tabs to the most recently added staff for the provided amount of times, each time
    /// until the guitar string position resets.
    pub fn add_spread_next(&mut self, amt: u32) {
        if let Some(staff) = self.staffs.last_mut() {
            staff.add_spread_next(amt);
        }
    }

    /// Sets global options on the staff manager based on the provided literal. Current
    /// and new staffs will have these options applied to them.
    /// 
    /// # Errors
    /// 
    /// This function errors if provided options contain syntax errors or unknown option names or values.
    pub fn set_options(&mut self, options: &str) -> Result<(), String> {
        self.options.set(options)
    }

    /// Creates a new staff with the current global options and appends it to the staff list.
    fn create_staff(&mut self) {
        let mut new_staff = Staff::new();
        // new staff will never have tabs so it is okay to unwrap values
        new_staff.set_time_signature(self.options.get_time_signature()).unwrap();
        new_staff.set_time_fidelity(self.options.get_time_fidelity()).unwrap();

        self.staffs.push(new_staff);
    }
}

impl fmt::Display for StaffManager {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut staffs = String::new();
        for staff in self.staffs.iter() {
            staffs.push_str(&(staff.to_string() + "\n"));
        }
        write!(f, "{}", staffs)
    }
}

/// Used for parsing the provided source `Vec<Token>` into an output string representing
/// guitar tablature notation.
/// 
/// # Examples
/// 
/// ```
/// use data::{Token, TokenType, Literal};
/// use parser::Parser;
/// 
/// let tokens = vec![
///     Token::new(TokenType::Note, String::from("E"), Literal::None, 1),
///     Token::new(TokenType::Note, String::from("A"), Literal::None, 1),
///     Token::new(TokenType::Note, String::from("D"), Literal::None, 1),
///     Token::new(TokenType::Note, String::from("G"), Literal::None, 1),
///     Token::new(TokenType::Note, String::from("B"), Literal::None, 1),
///     Token::new(TokenType::Note, String::from("E"), Literal::None, 1),
///     Token::new(TokenType::Number, String::from("0"), Literal::Number(0), 2),
///     Token::new(TokenType::Number, String::from("3"), Literal::Number(3), 2),
///     Token::new(TokenType::Number, String::from("5"), Literal::Number(5), 2),
///     Token::new(TokenType::EndOfFile, String::new(), Literal::None, 2),
/// ];
/// 
/// let mut parser = Parser::new(&tokens);
/// let tab_string = parser.generate_tabs();
/// 
/// match parser.generate_tabs() {
///     Ok(tab_string) => println!("Guitar tabs:\n\n{}", tab_string),
///     Err(e) => panic!("Could not generate tabs: {}", e),
/// }
/// ```
pub struct Parser<'a> {
    source: &'a Vec<Token>,
    tabs: String,
    watcher: Watcher,
}

impl<'a> Parser<'a> {
    /// Creates a new `Parser` for parsing through tokens and generating guitar tablature notation.
    pub fn new(source: &Vec<Token>) -> Parser {
        Parser {
            source,
            tabs: String::new(),
            watcher: Watcher::new(),
        }
    }

    /// Creates a string representing guitar tablature notation from the provided source tokens.
    pub fn generate_tabs(&mut self) -> Result<&str, String> {
        if self.tabs.is_empty() {
            // create a new staff manager to add token values to
            let mut staff_manager = StaffManager::new();

            for token in self.source.iter() {
                // check the token type and add to the staff manager based on type
                match token.type_of {
                    TokenType::Note => staff_manager.add_note(token.value.to_string()),
                    TokenType::Number => staff_manager.add_tab(&token.value),
                    TokenType::Empty => staff_manager.add_empty(),
                    TokenType::Next => staff_manager.add_next(),
                    TokenType::SpreadEmpty => {
                        if let Literal::Number(amt) = token.literal {
                            staff_manager.add_spread_empty(amt);
                        }
                    },
                    TokenType::SpreadNext => {
                        if let Literal::Number(amt) = token.literal {
                            staff_manager.add_spread_next(amt);
                        }
                    },
                    TokenType::Options => {
                        if let Literal::Options(ops) = &token.literal {
                            if let Err(e) = staff_manager.set_options(ops) {
                                self.watcher.error(token.line, format!("\n{}", e));
                            }
                        }
                    },
                    TokenType::EndOfFile => (),
                }
            }
            self.tabs = staff_manager.to_string();
        }

        // if there was a syntax error, return an error; otherwise return the token list
        if self.watcher.had_error {
            Err(self.watcher.to_string())
        } else {
            Ok(&self.tabs)
        }
    }
}

#[cfg(test)]
mod parser_tests {
    use super::*;

    #[test]
    fn tab_output() {
        let tokens = vec![
            Token::new(TokenType::Note, String::from("E"), Literal::None, 1),
            Token::new(TokenType::Note, String::from("A"), Literal::None, 1),
            Token::new(TokenType::Note, String::from("D"), Literal::None, 1),
            Token::new(TokenType::Note, String::from("G"), Literal::None, 1),
            Token::new(TokenType::Note, String::from("B"), Literal::None, 1),
            Token::new(TokenType::Note, String::from("E"), Literal::None, 1),
            Token::new(TokenType::Number, String::from("0"), Literal::Number(0), 2),
            Token::new(TokenType::Number, String::from("3"), Literal::Number(3), 2),
            Token::new(TokenType::Number, String::from("5"), Literal::Number(5), 2),
            Token::new(TokenType::Next, String::from(","), Literal::None, 2),
            Token::new(TokenType::EndOfFile, String::new(), Literal::None, 2),
        ];

        let mut parser = Parser::new(&tokens);
        let expected = String::from("E  |---\nB  |---\nG  |---\nD  |-5-\nA  |-3-\nE  |-0-\n\n     1 \n\n");

        match parser.generate_tabs() {
            Ok(found) => {
                assert_eq!(expected, found);
            },
            Err(e) => panic!("Could not generate tabs: {}", e),
        }
    }
}