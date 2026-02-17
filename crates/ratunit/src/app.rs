use junit_parser::TestSuites;

pub struct FileReport {
    pub filename: String,
    pub data: TestSuites,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum View {
    SuiteList,
    TestList,
    TestDetail,
}

pub struct App {
    pub files: Vec<FileReport>,
    pub selected_file: usize,
    pub selected_suite: usize,
    pub selected_test: usize,
    pub view: View,
    pub scroll_offset: u16,
    pub should_quit: bool,
    pub multi_file: bool,
}

impl App {
    pub fn new(files: Vec<FileReport>) -> Self {
        let multi_file = files.len() > 1;
        Self {
            files,
            selected_file: 0,
            selected_suite: 0,
            selected_test: 0,
            view: View::SuiteList,
            scroll_offset: 0,
            should_quit: false,
            multi_file,
        }
    }

    pub fn current_file(&self) -> &FileReport {
        &self.files[self.selected_file]
    }

    pub fn suite_count(&self) -> usize {
        self.current_file().data.suites.len()
    }

    pub fn test_count(&self) -> usize {
        if self.selected_suite < self.suite_count() {
            self.current_file().data.suites[self.selected_suite]
                .test_cases
                .len()
        } else {
            0
        }
    }

    pub fn select_next(&mut self) {
        match self.view {
            View::SuiteList => {
                let count = self.suite_count();
                if count > 0 && self.selected_suite < count - 1 {
                    self.selected_suite += 1;
                }
            }
            View::TestList => {
                let count = self.test_count();
                if count > 0 && self.selected_test < count - 1 {
                    self.selected_test += 1;
                }
            }
            View::TestDetail => {
                self.scroll_offset = self.scroll_offset.saturating_add(1);
            }
        }
    }

    pub fn select_prev(&mut self) {
        match self.view {
            View::SuiteList => {
                self.selected_suite = self.selected_suite.saturating_sub(1);
            }
            View::TestList => {
                self.selected_test = self.selected_test.saturating_sub(1);
            }
            View::TestDetail => {
                self.scroll_offset = self.scroll_offset.saturating_sub(1);
            }
        }
    }

    pub fn select_first(&mut self) {
        match self.view {
            View::SuiteList => self.selected_suite = 0,
            View::TestList => self.selected_test = 0,
            View::TestDetail => self.scroll_offset = 0,
        }
    }

    pub fn select_last(&mut self) {
        match self.view {
            View::SuiteList => {
                let count = self.suite_count();
                if count > 0 {
                    self.selected_suite = count - 1;
                }
            }
            View::TestList => {
                let count = self.test_count();
                if count > 0 {
                    self.selected_test = count - 1;
                }
            }
            View::TestDetail => {
                self.scroll_offset = u16::MAX / 2;
            }
        }
    }

    pub fn enter(&mut self) {
        match self.view {
            View::SuiteList => {
                if self.suite_count() > 0 {
                    self.selected_test = 0;
                    self.view = View::TestList;
                }
            }
            View::TestList => {
                if self.test_count() > 0 {
                    self.scroll_offset = 0;
                    self.view = View::TestDetail;
                }
            }
            View::TestDetail => {}
        }
    }

    pub fn go_back(&mut self) {
        match self.view {
            View::SuiteList => {}
            View::TestList => {
                self.view = View::SuiteList;
            }
            View::TestDetail => {
                self.view = View::TestList;
            }
        }
    }

    pub fn next_file(&mut self) {
        if self.multi_file {
            self.selected_file = (self.selected_file + 1) % self.files.len();
            self.reset_selection();
        }
    }

    pub fn prev_file(&mut self) {
        if self.multi_file {
            if self.selected_file == 0 {
                self.selected_file = self.files.len() - 1;
            } else {
                self.selected_file -= 1;
            }
            self.reset_selection();
        }
    }

    pub fn page_down(&mut self) {
        for _ in 0..10 {
            self.select_next();
        }
    }

    pub fn page_up(&mut self) {
        for _ in 0..10 {
            self.select_prev();
        }
    }

    fn reset_selection(&mut self) {
        self.selected_suite = 0;
        self.selected_test = 0;
        self.scroll_offset = 0;
        self.view = View::SuiteList;
    }

    pub fn aggregate_tests(&self) -> u64 {
        self.files.iter().map(|f| f.data.total_tests()).sum()
    }

    pub fn aggregate_passed(&self) -> u64 {
        self.files.iter().map(|f| f.data.total_passed()).sum()
    }

    pub fn aggregate_failures(&self) -> u64 {
        self.files.iter().map(|f| f.data.total_failures()).sum()
    }

    pub fn aggregate_errors(&self) -> u64 {
        self.files.iter().map(|f| f.data.total_errors()).sum()
    }

    pub fn aggregate_skipped(&self) -> u64 {
        self.files.iter().map(|f| f.data.total_skipped()).sum()
    }
}
