# BitSacco WhatsApp Bot Website - TypeScript Setup

This document explains the TypeScript setup for the BitSacco WhatsApp Bot static website.

## 🚀 Quick Start

### Prerequisites
- Node.js 18.0.0 or higher
- npm or yarn package manager

### Installation & Build

1. **Install dependencies:**
   ```bash
   npm install
   ```

2. **Build the project:**
   ```bash
   # For Unix/Linux/macOS
   ./build.sh
   
   # For Windows
   build.bat
   
   # Or use npm scripts
   npm run build
   ```

3. **Development mode (watch for changes):**
   ```bash
   npm run watch
   ```

## 📁 Project Structure

```
├── src/
│   └── script.ts          # Main TypeScript source file
├── dist/
│   └── script.js          # Compiled JavaScript (generated)
├── index.html             # Main HTML file
├── styles.css             # CSS styles
├── tsconfig.json          # TypeScript configuration
├── package.json           # Node.js dependencies and scripts
├── build.sh               # Unix build script
├── build.bat              # Windows build script
└── .gitignore             # Git ignore rules
```

## 🔧 TypeScript Configuration

The `tsconfig.json` file includes:
- **Strict mode** enabled for maximum type safety
- **ES2020** target for modern JavaScript features
- **DOM types** for browser APIs
- **Declaration files** generation for better IDE support
- **Source maps** for debugging

## 🎯 Key Features

### Type Safety
- **Interface definitions** for all data structures
- **Proper typing** for DOM elements and API responses
- **Error handling** with typed catch blocks
- **Null safety** with proper null checks

### Modern JavaScript Features
- **Async/await** for API calls
- **Template literals** for string formatting
- **Arrow functions** for concise code
- **Destructuring** for clean variable assignment

### Browser Compatibility
- **ES2020** target ensures compatibility with modern browsers
- **DOM API types** for safe element manipulation
- **Fetch API** with proper error handling

## 📝 Available Scripts

```bash
npm run build      # Compile TypeScript to JavaScript
npm run watch      # Watch for changes and recompile
npm run clean      # Remove build artifacts
npm run type-check # Check types without emitting files
```

## 🔍 Type Definitions

### Core Interfaces

```typescript
interface ChatMessage {
    user: string;
    bot: string;
}

interface BitcoinPrice {
    usd: number;
    kes: number;
    timestamp: string;
}

interface BitcoinPriceResponse {
    data: {
        amount: string;
        base: string;
        currency: string;
    };
}
```

### Utility Functions

```typescript
// Safe element selection with proper typing
function getElementById<T extends HTMLElement>(id: string): T | null

// Bitcoin price fetching with error handling
async function fetchBitcoinPrice(): Promise<void>

// Chat animation with proper state management
function animateBitSaccoChat(): void
```

## 🛠️ Development Workflow

1. **Edit TypeScript files** in the `src/` directory
2. **Run the build script** to compile to JavaScript
3. **Test the website** by opening `index.html` in a browser
4. **Use watch mode** during development for automatic recompilation

## 🚀 Deployment

The compiled JavaScript files in the `dist/` directory are ready for deployment. The website can be served from any static file server or deployed to platforms like:

- GitHub Pages
- Netlify
- Vercel
- AWS S3
- Any web server

## 🔧 Customization

### Adding New Features

1. **Define interfaces** for new data structures
2. **Add type-safe functions** with proper error handling
3. **Update the build process** if needed
4. **Test thoroughly** with different browsers

### Modifying Types

1. **Update interface definitions** in `src/script.ts`
2. **Ensure all usages** are properly typed
3. **Run type checking** with `npm run type-check`
4. **Rebuild the project** with `npm run build`

## 📚 Resources

- [TypeScript Handbook](https://www.typescriptlang.org/docs/)
- [DOM API Types](https://github.com/microsoft/TypeScript/tree/main/lib)
- [Modern JavaScript Features](https://developer.mozilla.org/en-US/docs/Web/JavaScript/Guide)

## 🤝 Contributing

When contributing to this project:

1. **Follow TypeScript best practices**
2. **Add proper type annotations**
3. **Include error handling**
4. **Test your changes thoroughly**
5. **Update documentation** as needed

## 📄 License

This project is licensed under the MIT License - see the main project README for details.
