# ratunit

A rat-powered TUI for viewing JUnit XML test reports.

## Install

### Homebrew

```
brew tap rupert648/tap
brew install ratunit
```

### From source

```
cargo install --git https://github.com/rupert648/ratunit.git ratunit
```

### Build locally

```
git clone https://github.com/rupert648/ratunit.git
cd ratunit
cargo install --path crates/ratunit
```

## Usage

```
ratunit test-reports/          # view a directory of XML files
ratunit report.xml             # view a single file
```

`j/k` navigate, `Enter` drill in, `Esc` back, `Tab` switch files, `q` quit.
