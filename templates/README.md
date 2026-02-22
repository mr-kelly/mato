# Mato State Templates

Pre-configured office templates for different use cases.

## Available Templates

### 1. Mato Creator Office (`power-user.json`) ⭐ RECOMMENDED
**Best for**: Serious professionals managing complex workflows

**Structure**: 20 desks with comprehensive coverage
- 🚀 Project X (15 tabs) - Main development with all AI tools
- 📱 Mobile App (13 tabs) - Mobile development
- 🌐 Web Platform (13 tabs) - Web development
- 💼 Marketing Hub (14 tabs) - Marketing operations
- 📊 Analytics Center (12 tabs) - Data analytics
- 💰 Revenue Ops (12 tabs) - Revenue operations
- 📝 Content Studio (12 tabs) - Content creation
- 🎨 Design Lab (12 tabs) - Design work
- 🎬 Video Production (12 tabs) - Video editing
- 🛠️ DevOps Control (12 tabs) - DevOps
- 🔒 Security Center (12 tabs) - Security
- ☁️ Cloud Infra (12 tabs) - Cloud infrastructure
- 🤖 AI Research (13 tabs) - AI research
- 🧠 ML Pipeline (12 tabs) - Machine learning
- 🔬 Prompt Lab (12 tabs) - Prompt engineering
- 📞 Customer Support (12 tabs) - Support
- 🎓 Knowledge Base (12 tabs) - Documentation
- 🎯 Strategy (12 tabs) - Strategic planning
- 🔬 Research2 (12 tabs) - Research
- 💼 Business Ops (12 tabs) - Business operations
**Total**: 20 desks, 248 tabs

**Features**:
- All major AI coding assistants (Claude, Gemini, Codex, Copilot, Cursor, Aider, Continue, Cline, Windsurf, Bolt)
- Complete business function coverage
- Ready for enterprise-level workflows
- Emoji-based visual organization

---

### 2. Solo Developer (`solo-developer.json`)
**Best for**: Individual developers working on a single project

**Structure**:
- 💻 Development (3 tabs)
  - 📝 Editor
  - 🚀 Dev Server
  - 📋 Logs
- 🔀 Git & Deploy (2 tabs)
  - 🌿 Git
  - 🚢 Deploy
- 🛠️ Tools (3 tabs)
  - 🗄️ Database
  - 🐳 Docker
  - 📊 Monitor

**Total**: 3 desks, 8 tabs

---

### 3. One-Person Company (`one-person-company.json`)
**Best for**: Solo entrepreneurs managing multiple business functions

**Structure**:
- 🔧 Engineering (4 tabs)
  - 🎨 Frontend
  - ⚙️ Backend
  - ☁️ Infrastructure
  - 🧪 Testing
- 📦 Product (3 tabs)
  - 🎨 Design
  - 📚 Documentation
  - 🔍 Research
- 📢 Marketing (3 tabs)
  - ✍️ Content
  - 📱 Social Media
  - 📊 Analytics
- ⚡ Operations (3 tabs)
  - 💬 Support
  - 💰 Finance
  - 📋 Admin

**Total**: 4 desks, 13 tabs

---

### 4. Full-Stack Developer (`fullstack-developer.json`)
**Best for**: Developers working on multiple projects

**Structure**:
- 🚀 Main Project (4 tabs)
  - 💻 Code
  - 🎨 Frontend Dev
  - ⚙️ Backend Dev
  - 🧪 Tests
- 💡 Side Project (2 tabs)
  - 💻 Code
  - 🚀 Server
- 🔧 DevOps (3 tabs)
  - 🐳 Docker
  - ☸️ Kubernetes
  - 🔄 CI/CD
- 📚 Learning (2 tabs)
  - 📖 Tutorial
  - 🧪 Experiments

**Total**: 4 desks, 11 tabs

---

### 5. Data Scientist (`data-scientist.json`)
**Best for**: Data scientists and ML engineers

**Structure**:
- 📊 Data Analysis (3 tabs)
  - 📓 Jupyter
  - 🐍 Python
  - 📈 Visualization
- 🤖 ML Training (3 tabs)
  - 🏋️ Training
  - 📊 TensorBoard
  - 🎮 GPU Monitor
- 🔄 Data Pipeline (3 tabs)
  - 🔧 ETL
  - 🌊 Airflow
  - 🗄️ Database
- 🚀 Deployment (2 tabs)
  - 🔌 API
  - 🐳 Docker

**Total**: 4 desks, 11 tabs

---

### 6. Minimal (`minimal.json`)
**Best for**: Starting from scratch

**Structure**:
- Task 1 (1 tab)
  - Terminal 1

**Total**: 1 task, 1 tab

---

## How to Use Templates

### Method 1: First-Time Setup (Automatic)

When you first run Mato without any existing state, you'll be prompted to choose a template:

```bash
mato
```

You'll see:
```
Welcome to Mato! 🎉

Choose a office template:
1. Mato Creator Office (20 desks, 248 tabs) ⭐ RECOMMENDED
2. Solo Developer (3 desks, 8 tabs)
3. One-Person Company (4 desks, 13 tabs)
4. Full-Stack Developer (4 desks, 11 tabs)
5. Data Scientist (4 desks, 11 tabs)
6. Minimal (1 task, 1 tab)

Enter your choice (1-6):
```

Language selection:
- Use `←` / `→` in onboarding to switch language.
- Supported languages: English, Simplified Chinese, Traditional Chinese, Japanese, Korean.

### Method 2: Manual Setup

Copy a template to your state file:

```bash
# Choose a template
cp templates/solo-developer.json ~/.config/mato/state.json

# Start Mato
mato
```

### Method 3: Reset to Template

If you want to reset your office:

```bash
# Backup current state (optional)
cp ~/.config/mato/state.json ~/.config/mato/state.json.backup

# Apply template
cp templates/fullstack-developer.json ~/.config/mato/state.json

# Restart Mato
mato
```

## Creating Custom Templates

You can create your own templates:

1. Set up your ideal office in Mato
2. Copy your state file:
   ```bash
   cp ~/.config/mato/state.json my-custom-template.json
   ```
3. Edit the file to customize names and structure
4. Share with others or keep for personal use

### Template Format

```json
{
  "desks": [
    {
      "id": "unique-task-id",
      "name": "Task Name",
      "tabs": [
        {
          "id": "unique-tab-id",
          "name": "Tab Name"
        }
      ],
      "active_tab": 0
    }
  ],
  "active_task": 0
}
```

**Important**:
- Each `id` must be unique
- `active_tab` is 0-indexed (0 = first tab)
- `active_task` is 0-indexed (0 = first task)

## Tips

### Emoji in Names
Templates use emoji for visual clarity. You can:
- Keep them for a colorful office
- Remove them for a minimal look
- Replace with your own emoji

### Organizing Desks
Common organization patterns:
- **By Project**: Main, Side, Experiments
- **By Function**: Dev, Ops, Marketing
- **By Stage**: Planning, Development, Testing, Deployment
- **By Technology**: Frontend, Backend, Database, DevOps

### Tab Naming
Good tab names are:
- Short and descriptive
- Use emoji for quick recognition
- Consistent across similar desks

## Examples in Action

### Solo Developer Workflow
```
1. Start in "Development" task
2. Alt+1: Editor (write code)
3. Alt+2: Dev Server (check output)
4. Alt+3: Logs (debug issues)
5. Switch to "Git & Deploy" task
6. Alt+1: Git (commit changes)
7. Alt+2: Deploy (push to production)
```

### One-Person Company Workflow
```
Morning: Engineering
- Frontend work
- Backend API updates

Afternoon: Marketing
- Write blog post
- Schedule social media

Evening: Operations
- Check support tickets
- Review finances
```

## Contributing Templates

Have a great template? Share it!

1. Create your template JSON file
2. Add description to this README
3. Submit a pull request

Popular templates will be included in future releases.

## See Also

- [README.md](../README.md) - Main documentation
- [KEYBOARD_SHORTCUTS.md](KEYBOARD_SHORTCUTS.md) - Keyboard shortcuts
- [CONFIGURATION.md](CONFIGURATION.md) - Configuration options
