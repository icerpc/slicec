
//------------------------------------------------------------------------------
// Location
//------------------------------------------------------------------------------
#[derive(Clone, Debug)]
pub struct Location {
    pub start: (usize, usize),
    pub end: (usize, usize),
    pub file: String,
}

//------------------------------------------------------------------------------
// error reporting function
//------------------------------------------------------------------------------
pub fn report_issue(prefix: &str, message: &str, location: Option<Location>) {
    // TODO report the errors!
}

//------------------------------------------------------------------------------
// error reporting macros
//------------------------------------------------------------------------------
macro_rules! issue_error{
    ($a:expr) => {
        report_issue("error", $a, None);
    };
    ($a:expr, $b: expr) => {
        report_issue("error", $a, Some($b));
    };
}

macro_rules! issue_warning{
    ($a:expr) => {
        report_issue("warning", $a, None);
    };
    ($a:expr, $b: expr) => {
        report_issue("warning", $a, Some($b));
    };
}

macro_rules! issue_note{
    ($a:expr) => {
        report_issue("note", $a, None);
    };
    ($a:expr, $b: expr) => {
        report_issue("note", $a, Some($b));
    };
}

pub fn testing() {
    let myLoc = Location { start: (0,0), end: (8,8), file: "hello".to_owned()};
    issue_error!("hello!", myLoc);
    issue_warning!("hello!");
    issue_note!("hello!");
}