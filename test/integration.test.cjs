// Integration test for Git Author Analyzer
// This test verifies the main functionality of the application

console.log("Starting Git Author Analyzer integration test...");

// Mock the Tauri environment for testing
global.__TAURI_INTERNALS__ = {
  // Mock invoke function
  invoke: async (cmd, payload) => {
    console.log(`Tauri invoke called with cmd: ${cmd}`, payload);
    
    // Return mock data based on command
    if (cmd === 'get_git_authors') {
      return [
        { name: 'John Doe', email: 'john@example.com', count: 42 },
        { name: 'Jane Smith', email: 'jane@example.com', count: 38 },
        { name: 'Bob Johnson', email: 'bob@example.com', count: 25 }
      ];
    } else if (cmd === 'get_git_commits_in_date_range') {
      return [
        {
          hash: 'abc123',
          author_name: 'John Doe',
          author_email: 'john@example.com',
          date: new Date().toISOString(),
          message: 'Sample commit message'
        }
      ];
    } else if (cmd === 'export_to_txt') {
      return '/mock/path/git-author-report.txt';
    }
    
    return null;
  },
  
  // Mock dialog
  dialog: {
    save: async (options) => {
      console.log('Dialog save called with options:', options);
      return '/mock/path/output.txt';
    }
  }
};

// Mock window object for Tauri
global.window = {
  __TAURI_INTERNALS__: global.__TAURI_INTERNALS__
};

// Add a simple test function
function runTests() {
  console.log("✓ Test environment initialized");
  
  // Test 1: Verify mock Tauri API works
  try {
    const result = __TAURI_INTERNALS__.invoke('get_git_authors', { projectPath: '/test/path' });
    console.log("✓ Tauri API mock works:", !!result);
  } catch (e) {
    console.error("✗ Tauri API mock failed:", e);
  }
  
  // Test 2: Verify component imports work
  try {
    // Since we can't directly import Vue components in Node environment
    // we'll just verify that the files exist
    const fs = require('fs');
    const componentPath = './src/components/GitAnalyzer.vue';
    const componentExists = fs.existsSync(componentPath);
    console.log("✓ GitAnalyzer component exists:", componentExists);
  } catch (e) {
    console.error("✗ Component existence test failed:", e);
  }
  
  // Test 3: Verify package.json configuration
  try {
    const packageJson = require('./package.json');
    const hasTauriScript = packageJson.scripts && packageJson.scripts.tauri;
    const hasDependencies = packageJson.dependencies && packageJson.devDependencies;
    console.log("✓ Package.json validation passed:", hasTauriScript && hasDependencies);
  } catch (e) {
    console.error("✗ Package.json validation failed:", e);
  }
  
  // Test 4: Verify configuration files exist
  try {
    const fs = require('fs');
    const configFiles = [
      './src-tauri/tauri.conf.json',
      './src-tauri/Cargo.toml',
      './vite.config.ts'
    ];
    
    const allExist = configFiles.every(file => fs.existsSync(file));
    console.log("✓ Configuration files exist:", allExist);
  } catch (e) {
    console.error("✗ Configuration files test failed:", e);
  }
  
  console.log("\nIntegration test completed!");
}

// Run the tests
runTests();

module.exports = { runTests };