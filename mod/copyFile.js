const fs = require('fs');
const path = require('path');

// File names and paths
const fileName = 'save-sync-integration.mod.zip';
const sourcePath = path.join(__dirname, fileName);
const destDir = path.join(__dirname, '..', 'server', 'res');
const destPath = path.join(destDir, fileName);

// Make sure the destination directory exists
try {
  if (!fs.existsSync(destDir)) {
    fs.mkdirSync(destDir, { recursive: true });
    console.log(`Created directory: ${destDir}`);
  }
} catch (err) {
  console.error(`Error creating directory: ${err.message}`);
  process.exit(1);
}

// Check if file exists at destination
const fileExistsAtDest = fs.existsSync(destPath);

// Copy the file
try {
  fs.copyFileSync(sourcePath, destPath);

  if (fileExistsAtDest) {
    console.log(`File ${fileName} successfully overwritten at ${destPath}`);
  } else {
    console.log(`File ${fileName} successfully copied to ${destPath}`);
  }
} catch (err) {
  console.error(`Error copying file: ${err.message}`);
  process.exit(1);
}
