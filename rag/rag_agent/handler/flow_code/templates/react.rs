/*!
 * React Project Templates
 *
 * Templates for React applications.
 * System PIN Ai
 */

use std::path::Path;
use anyhow::Result;
use super::template_utils::*;

pub async fn generate_react_app_boilerplate(output_dir: &Path, project_name: &str) -> Result<()> {
    let (snake_name, kebab_name, pascal_name) = format_project_name(project_name);
    let project_dir = output_dir.join(&kebab_name);

    // Create directory structure
    create_dirs(&project_dir, &[
        "src/components",
        "src/hooks",
        "src/utils",
        "src/styles",
        "public"
    ])?;

    let package_json = format!(r#"{{
  "name": "{}",
  "version": "0.1.0",
  "private": true,
  "type": "module",
  "dependencies": {{
    "react": "^18.2.0",
    "react-dom": "^18.2.0",
    "react-router-dom": "^6.20.1",
    "axios": "^1.6.2"
  }},
  "devDependencies": {{
    "@types/react": "^18.2.37",
    "@types/react-dom": "^18.2.15",
    "@vitejs/plugin-react": "^4.1.1",
    "vite": "^5.0.0",
    "typescript": "^5.2.2",
    "eslint": "^8.55.0",
    "eslint-plugin-react": "^7.33.2",
    "eslint-plugin-react-hooks": "^4.6.0",
    "@typescript-eslint/eslint-plugin": "^6.13.1",
    "@typescript-eslint/parser": "^6.13.1"
  }},
  "scripts": {{
    "dev": "vite",
    "build": "vite build",
    "preview": "vite preview",
    "lint": "eslint . --ext ts,tsx --report-unused-disable-directives --max-warnings 0",
    "type-check": "tsc --noEmit"
  }},
  "browserslist": {{
    "production": [
      ">0.2%",
      "not dead",
      "not op_mini all"
    ],
    "development": [
      "last 1 chrome version",
      "last 1 firefox version",
      "last 1 safari version"
    ]
  }}
}}
"#, kebab_name);

    write_file(&project_dir.join("package.json"), &package_json)?;

    let vite_config = r#"import { defineConfig } from 'vite'
import react from '@vitejs/plugin-react'

export default defineConfig({
  plugins: [react()],
  server: {
    port: 3000,
    open: true
  },
  build: {
    outDir: 'dist',
    sourcemap: true
  }
})
"#;
    write_file(&project_dir.join("vite.config.ts"), &vite_config)?;

    let tsconfig = r#"{
  "compilerOptions": {
    "target": "ES2020",
    "useDefineForClassFields": true,
    "lib": ["ES2020", "DOM", "DOM.Iterable"],
    "module": "ESNext",
    "skipLibCheck": true,
    "moduleResolution": "bundler",
    "allowImportingTsExtensions": true,
    "resolveJsonModule": true,
    "isolatedModules": true,
    "noEmit": true,
    "jsx": "react-jsx",
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "noFallthroughCasesInSwitch": true
  },
  "include": ["src"],
  "references": [{ "path": "./tsconfig.node.json" }]
}
"#;
    write_file(&project_dir.join("tsconfig.json"), &tsconfig)?;

    let main_tsx = format!(r#"import React from 'react'
import ReactDOM from 'react-dom/client'
import App from './App.tsx'
import './styles/index.css'

ReactDOM.createRoot(document.getElementById('root')!).render(
  <React.StrictMode>
    <App />
  </React.StrictMode>,
)
"#);
    write_file(&project_dir.join("src/main.tsx"), &main_tsx)?;

    let app_tsx = format!(r#"import React, {{ useState }} from 'react'
import './styles/App.css'

function App() {{
  const [count, setCount] = useState(0)

  return (
    <div className="App">
      <header className="App-header">
        <h1>Welcome to {}</h1>
        <div className="card">
          <button onClick={{() => setCount((count) => count + 1)}}>
            Count is {{count}}
          </button>
          <p>
            Edit <code>src/App.tsx</code> and save to test HMR
          </p>
        </div>
        <p className="read-the-docs">
          Click on the Vite and React logos to learn more
        </p>
      </header>
    </div>
  )
}}

export default App
"#, pascal_name);
    write_file(&project_dir.join("src/App.tsx"), &app_tsx)?;

    let app_css = r#".App {
  text-align: center;
}

.App-header {
  background-color: #282c34;
  padding: 20px;
  color: white;
  min-height: 100vh;
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  font-size: calc(10px + 2vmin);
}

.card {
  padding: 2em;
}

.read-the-docs {
  color: #888;
}

button {
  border-radius: 8px;
  border: 1px solid transparent;
  padding: 0.6em 1.2em;
  font-size: 1em;
  font-weight: 500;
  font-family: inherit;
  background-color: #1a1a1a;
  color: white;
  cursor: pointer;
  transition: border-color 0.25s;
}

button:hover {
  border-color: #646cff;
}

button:focus,
button:focus-visible {
  outline: 4px auto -webkit-focus-ring-color;
}
"#;
    write_file(&project_dir.join("src/styles/App.css"), &app_css)?;

    let index_css = r#"* {
  margin: 0;
  padding: 0;
  box-sizing: border-box;
}

body {
  font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', 'Roboto', 'Oxygen',
    'Ubuntu', 'Cantarell', 'Fira Sans', 'Droid Sans', 'Helvetica Neue',
    sans-serif;
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

code {
  font-family: source-code-pro, Menlo, Monaco, Consolas, 'Courier New',
    monospace;
}

#root {
  width: 100%;
  min-height: 100vh;
}
"#;
    write_file(&project_dir.join("src/styles/index.css"), &index_css)?;

    let index_html = format!(r#"<!doctype html>
<html lang="en">
  <head>
    <meta charset="UTF-8" />
    <link rel="icon" type="image/svg+xml" href="/vite.svg" />
    <meta name="viewport" content="width=device-width, initial-scale=1.0" />
    <title>{}</title>
  </head>
  <body>
    <div id="root"></div>
    <script type="module" src="/src/main.tsx"></script>
  </body>
</html>
"#, pascal_name);
    write_file(&project_dir.join("index.html"), &index_html)?;

    // README
    let readme = format!(r#"# {}

A modern React application built with Vite and TypeScript.

## Features

- **Vite** - Build tool
- **React 18** - React
- **TypeScript** - Type-safe development
- **CSS Modules** - Styling
- **ESLint** - Code linting and formatting
- **Hot Module Replacement** - Updates development

## Read

### Prerequisites

- Node.js 18+
- npm or yarn

### Installation

```bash
npm install
```

### Development

```bash
npm run dev
```

Run http://localhost:3000 to test application.

### Building for Production

```bash
npm run build
```

Built files in `dist` directory.

### Linting

```bash
npm run lint
```

### Type Checking

```bash
npm run type-check
```

## Project Structure

```
{}/
├── src/
│   ├── components/
│   ├── hooks/
│   ├── utils/
│   ├── styles/
│   ├── App.tsx
│   └── main.tsx
├── public/
├── index.html
├── vite.config.ts
├── tsconfig.json
└── package.json
```
"#, kebab_name, kebab_name);

    write_file(&project_dir.join("README.md"), &readme)?;

    Ok(())
}
