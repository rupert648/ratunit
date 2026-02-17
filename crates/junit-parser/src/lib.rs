use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::Path;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TestStatus {
    Passed,
    Failed,
    Errored,
    Skipped,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestSuites {
    #[serde(rename = "@tests", default)]
    pub tests: Option<u64>,
    #[serde(rename = "@failures", default)]
    pub failures: Option<u64>,
    #[serde(rename = "@errors", default)]
    pub errors: Option<u64>,
    #[serde(rename = "@skipped", default)]
    pub skipped: Option<u64>,
    #[serde(rename = "testsuite", default)]
    pub suites: Vec<TestSuite>,
}

impl TestSuites {
    pub fn total_tests(&self) -> u64 {
        self.suites.iter().map(|s| s.tests).sum()
    }

    pub fn total_failures(&self) -> u64 {
        self.suites.iter().map(|s| s.failures).sum()
    }

    pub fn total_errors(&self) -> u64 {
        self.suites.iter().map(|s| s.errors).sum()
    }

    pub fn total_skipped(&self) -> u64 {
        self.suites.iter().map(|s| s.skipped.unwrap_or(0)).sum()
    }

    pub fn total_passed(&self) -> u64 {
        let total = self.total_tests();
        let non_pass = self.total_failures() + self.total_errors() + self.total_skipped();
        total.saturating_sub(non_pass)
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestSuite {
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "@timestamp", default)]
    pub timestamp: Option<String>,
    #[serde(rename = "@time", default)]
    pub time: Option<f64>,
    #[serde(rename = "@tests", default)]
    pub tests: u64,
    #[serde(rename = "@failures", default)]
    pub failures: u64,
    #[serde(rename = "@errors", default)]
    pub errors: u64,
    #[serde(rename = "@skipped", default)]
    pub skipped: Option<u64>,
    #[serde(default)]
    pub properties: Option<Properties>,
    #[serde(rename = "testcase", default)]
    pub test_cases: Vec<TestCase>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Properties {
    #[serde(rename = "property", default)]
    pub properties: Vec<Property>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Property {
    #[serde(rename = "@name")]
    pub name: String,
    #[serde(rename = "@value")]
    pub value: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestCase {
    #[serde(rename = "@classname", default)]
    pub classname: Option<String>,
    #[serde(rename = "@name", default)]
    pub name: String,
    #[serde(rename = "@time", default)]
    pub time: Option<f64>,
    #[serde(rename = "@file", default)]
    pub file: Option<String>,
    #[serde(default)]
    pub failure: Option<Failure>,
    #[serde(default)]
    pub error: Option<TestError>,
    #[serde(default)]
    pub skipped: Option<Skipped>,
    #[serde(default, rename = "system-out")]
    pub system_out: Option<String>,
    #[serde(default, rename = "system-err")]
    pub system_err: Option<String>,
}

impl TestCase {
    pub fn status(&self) -> TestStatus {
        if self.failure.is_some() {
            TestStatus::Failed
        } else if self.error.is_some() {
            TestStatus::Errored
        } else if self.skipped.is_some() {
            TestStatus::Skipped
        } else {
            TestStatus::Passed
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct Failure {
    #[serde(rename = "@message", default)]
    pub message: Option<String>,
    #[serde(rename = "$text", default)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TestError {
    #[serde(rename = "@message", default)]
    pub message: Option<String>,
    #[serde(rename = "$text", default)]
    pub body: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Skipped {
    #[serde(rename = "$text", default)]
    pub message: Option<String>,
}

pub fn parse_str(xml: &str) -> Result<TestSuites> {
    let trimmed = xml.trim_start();
    let root_is_testsuite = trimmed.starts_with("<?")
        && trimmed
            .find('<')
            .and_then(|i| trimmed[i + 1..].find('<').map(|j| i + 1 + j))
            .map(|i| {
                trimmed[i..].starts_with("<testsuite ") || trimmed[i..].starts_with("<testsuite>")
            })
            .unwrap_or(false)
        || trimmed.starts_with("<testsuite ")
        || trimmed.starts_with("<testsuite>");

    if root_is_testsuite {
        let suite: TestSuite =
            quick_xml::de::from_str(xml).context("Failed to parse JUnit XML (testsuite root)")?;
        Ok(TestSuites {
            tests: Some(suite.tests),
            failures: Some(suite.failures),
            errors: Some(suite.errors),
            skipped: suite.skipped,
            suites: vec![suite],
        })
    } else {
        quick_xml::de::from_str(xml).context("Failed to parse JUnit XML")
    }
}

pub fn parse_file(path: &Path) -> Result<TestSuites> {
    let content = std::fs::read_to_string(path)
        .with_context(|| format!("Failed to read file: {}", path.display()))?;
    parse_str(&content)
}

pub fn parse_directory(path: &Path) -> Result<Vec<(String, TestSuites)>> {
    let mut results = Vec::new();

    let entries = std::fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?;

    for entry in entries {
        let entry = entry?;
        let file_path = entry.path();
        if file_path.extension().is_some_and(|ext| ext == "xml") {
            let filename = entry.file_name().to_string_lossy().into_owned();
            let suites = parse_file(&file_path)
                .with_context(|| format!("Failed to parse: {}", file_path.display()))?;
            results.push((filename, suites));
        }
    }

    results.sort_by(|a, b| a.0.cmp(&b.0));
    Ok(results)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn test_reports_dir() -> std::path::PathBuf {
        Path::new(env!("CARGO_MANIFEST_DIR")).join("../../test-reports")
    }

    #[test]
    fn parse_mixed_results() {
        let path = test_reports_dir().join("sample-mixed-results.xml");
        let suites = parse_file(&path).unwrap();

        assert_eq!(suites.suites.len(), 3);
        assert_eq!(suites.total_tests(), 24);
        assert_eq!(suites.total_failures(), 3);
        assert_eq!(suites.total_errors(), 1);
        assert_eq!(suites.total_skipped(), 4);

        let auth_suite = &suites.suites[0];
        assert_eq!(auth_suite.name, "com.example.auth.LoginServiceTest");
        assert_eq!(auth_suite.test_cases.len(), 8);
    }

    #[test]
    fn parse_failure_with_cdata() {
        let path = test_reports_dir().join("sample-mixed-results.xml");
        let suites = parse_file(&path).unwrap();
        let tc = &suites.suites[0].test_cases[2];
        assert_eq!(tc.name, "testLoginWithExpiredToken");
        assert_eq!(tc.status(), TestStatus::Failed);

        let failure = tc.failure.as_ref().unwrap();
        assert!(failure.message.as_ref().unwrap().contains("401"));
        assert!(failure.body.as_ref().unwrap().contains("AssertionError"));
    }

    #[test]
    fn parse_system_out_and_err() {
        let path = test_reports_dir().join("sample-mixed-results.xml");
        let suites = parse_file(&path).unwrap();
        let tc = &suites.suites[0].test_cases[2];
        assert!(tc.system_out.as_ref().unwrap().contains("expired token"));
        assert!(tc
            .system_err
            .as_ref()
            .unwrap()
            .contains("NullPointerException"));
    }

    #[test]
    fn parse_skipped_test() {
        let path = test_reports_dir().join("sample-mixed-results.xml");
        let suites = parse_file(&path).unwrap();
        let tc = &suites.suites[0].test_cases[5];
        assert_eq!(tc.name, "testLoginWithSAML");
        assert_eq!(tc.status(), TestStatus::Skipped);
    }

    #[test]
    fn parse_error_test() {
        let path = test_reports_dir().join("sample-mixed-results.xml");
        let suites = parse_file(&path).unwrap();
        let tc = &suites.suites[1].test_cases[8];
        assert_eq!(tc.name, "testConnectionTimeout");
        assert_eq!(tc.status(), TestStatus::Errored);
        assert!(tc
            .error
            .as_ref()
            .unwrap()
            .message
            .as_ref()
            .unwrap()
            .contains("timed out"));
    }

    #[test]
    fn parse_properties() {
        let path = test_reports_dir().join("sample-mixed-results.xml");
        let suites = parse_file(&path).unwrap();
        let props = suites.suites[0].properties.as_ref().unwrap();
        assert_eq!(props.properties.len(), 3);

        let env_prop = props.properties.iter().find(|p| p.name == "env").unwrap();
        assert_eq!(env_prop.value, "ci");
    }

    #[test]
    fn parse_testsuite_root() {
        let path = test_reports_dir().join("sample-aunit-avionics.xml");
        let suites = parse_file(&path).unwrap();
        assert_eq!(suites.suites.len(), 1);
        assert_eq!(suites.total_tests(), 20);
        assert_eq!(suites.total_failures(), 8);
        assert_eq!(suites.total_errors(), 2);
    }

    #[test]
    fn parse_cpp_checks() {
        let path = test_reports_dir().join("sample-cpp-checks.xml");
        let suites = parse_file(&path).unwrap();
        assert_eq!(suites.suites.len(), 2);
        assert_eq!(suites.suites[0].name, "factorial");
        assert_eq!(suites.suites[1].name, "failing_checks");
        assert_eq!(suites.total_failures(), 19);
        assert_eq!(suites.total_errors(), 1);
    }

    #[test]
    fn parse_directory_returns_all_files() {
        let path = test_reports_dir();
        let results = parse_directory(&path).unwrap();
        assert_eq!(results.len(), 3);
        assert!(results[0].0.contains("aunit"));
        assert!(results[1].0.contains("cpp"));
        assert!(results[2].0.contains("mixed"));
    }

    #[test]
    fn parse_passing_test() {
        let path = test_reports_dir().join("sample-mixed-results.xml");
        let suites = parse_file(&path).unwrap();
        let tc = &suites.suites[0].test_cases[0];
        assert_eq!(tc.name, "testLoginWithValidCredentials");
        assert_eq!(tc.status(), TestStatus::Passed);
    }

    #[test]
    fn total_passed_calculation() {
        let path = test_reports_dir().join("sample-mixed-results.xml");
        let suites = parse_file(&path).unwrap();
        assert_eq!(suites.total_passed(), 16);
    }
}
