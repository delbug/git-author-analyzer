# Git Author Analyzer

A desktop application built with Tauri, Vue 3, TypeScript, and Tailwind CSS to analyze Git repositories and export author contribution reports.

## Features

- Load and display all contributors from a Git repository
- Select one or multiple authors to analyze
- Filter contributions by:
  - Single date
  - Multiple dates
  - Date ranges
- Export detailed reports to TXT files

## Technologies Used

- **Frontend**: Vue 3 + TypeScript
- **Styling**: Tailwind CSS
- **Desktop Framework**: Tauri (Rust-based)
- **Backend**: Rust with git2 library for Git operations
- **Build Tool**: Vite

## Getting Started

1. **Prerequisites**
   - Node.js (v16 or higher)
   - Rust (latest stable)
   - Git

2. **Installation**
   ```bash
   npm install
   ```

3. **Development**
   ```bash
   # Run the development server
   npm run dev
   
   # Run the Tauri application in development mode
   npm run tauri dev
   ```

4. **Build**
   ```bash
   # Build the application
   npm run tauri build
   ```

## How to Use

1. Launch the application
2. Enter the path to your local Git repository
3. Click "Load Authors" to retrieve the list of contributors
4. Select one or more authors using the checkboxes
5. Choose your desired date filtering options:
   - Single date
   - Multiple dates
   - Date ranges
6. Click "Export to TXT" to generate and download the report

## Testing

To run the automated tests:

```bash
node test/run-tests.cjs
```

This runs integration tests that verify:
- Application structure integrity
- Dependency configurations
- Component availability
- Rust code validity

## Architecture

The application consists of:

- **Frontend**: Vue 3 components in `src/components/`
- **Backend**: Rust commands in `src-tauri/src/lib.rs`
- **Configuration**: Tauri settings in `src-tauri/tauri.conf.json`
- **Styling**: Tailwind CSS configured in `tailwind.config.js`

## License

MIT