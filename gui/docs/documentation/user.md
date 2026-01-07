# Phase 11.2: User Documentation

## Overview

**Component**: User Documentation
**Purpose**: End-user documentation for Rustica OS desktop environment
**Format**: Markdown (mdBook)
**Location**: `/var/www/rustux.com/prod/apps/gui/docs/user/`

## Goals

1. **User-Friendly**: Easy to understand for all skill levels
2. **Comprehensive**: Cover all features and functionality
3. **Visual**: Include screenshots and diagrams
4. **Searchable**: Easy to find information
5. **Translatable**: Support multiple languages

## Documentation Structure

```
docs/user/
├── README.md                    # User documentation overview
├── getting-started/
│   ├── introduction.md          # Welcome to Rustica OS
│   ├── first-steps.md          # Initial setup guide
│   ├── desktop-tour.md          # Desktop environment tour
│   └── basic-usage.md           # Basic operations
├── desktop/
│   ├── panel.md                 # Top panel usage
│   ├── dock.md                  # Dock usage
│   ├── workspaces.md            # Workspace management
│   ├── notifications.md         # Notification center
│   └── launcher.md              # App launcher
├── applications/
│   ├── files.md                 # File manager
│   ├── terminal.md              # Terminal emulator
│   ├── settings.md              # Settings app
│   ├── text-editor.md           # Text editor
│   ├── screenshot.md            # Screenshot tool
│   └── app-library.md           # Application library
├── customization/
│   ├── appearance.md            # Appearance settings
│   ├── themes.md                # Using themes
│   ├── keyboard-shortcuts.md    # Keyboard shortcuts
│   ├── gestures.md              # Touch gestures
│   └── extensions.md            # Extensions and plugins
├── mobile/
│   ├── touch-basics.md          # Touch basics
│   ├── gestures.md              # Touch gestures
│   ├── keyboard.md              # On-screen keyboard
│   ├── battery.md               # Battery optimization
│   └── rotation.md              # Screen rotation
├── accessibility/
│   ├── screen-reader.md         # Screen reader usage
│   ├── magnifier.md             # Screen magnifier
│   ├── high-contrast.md         # High contrast mode
│   ├── keyboard-nav.md          # Keyboard navigation
│   └── voice-control.md         # Voice control
└── troubleshooting/
    ├── common-issues.md         # Common problems
    ├── hardware.md              # Hardware issues
    ├── apps.md                  # Application issues
    └── recovery.md              # System recovery
```

## Getting Started Guide

```markdown
# Welcome to Rustica OS

## Introduction

Rustica OS is a modern, privacy-focused Linux distribution built from the ground up with Rust. It features a beautiful, intuitive desktop environment designed for both desktop and mobile devices.

![Rustica Desktop](../images/desktop-screenshot.png)

## What Makes Rustica Different?

- **Built with Rust**: Memory-safe and secure from the kernel to the GUI
- **Privacy First**: No telemetry, no tracking, you're in control
- **Modern Design**: Beautiful Material Design 3-inspired interface
- **Touch-Ready**: Works seamlessly on tablets and touch devices
- **Lightweight**: Fast and responsive, even on older hardware

## First Time Setup

When you first boot Rustica OS, you'll be guided through the initial setup:

1. **Welcome**: Click "Get Started" to begin
2. **Account**: Create your user account
3. **Network**: Connect to WiFi (optional)
4. **Privacy**: Choose your privacy settings
5. **Customization**: Pick your theme and wallpaper
6. **Complete**: Click "Start Using Rustica"

## Desktop Tour

### The Panel

The top panel provides quick access to system features:

```
┌─────────────────────────────────────────────────────────────┐
│ [Activities] [Window] Firefox |  🔊  🔋  👤  📅  ⋮       │
└─────────────────────────────────────────────────────────────┘
    Menu        Apps    Running  Vol  Bat User Date  System
```

- **Activities**: Open the app launcher and workspace overview
- **Window**: Show active window menus
- **App Indicator**: Running apps show indicators here
- **Status Icons**: Volume, battery, network, etc.
- **System Menu**: Access settings, power options, etc.

### The Dock

The dock on the left provides quick access to your favorite apps:

```
┌┐
││ 🌐  Firefox
│ │ 📁 Files
│ │ ⚙️  Settings
│ │ 💻 Terminal
│ │ ···
│ │ □ Show Apps
└┘
```

- Click an icon to launch the app
- Right-click for app options
- Drag to reorder
- Drag new apps to add them

### Workspaces

Rustica OS uses dynamic workspaces:

- **Overview**: Press Super or click "Activities" to see all workspaces
- **Add**: Click "+" to add a new workspace
- **Switch**: Click a workspace or use Super+PgUp/PgDn
- **Move Windows**: Drag windows between workspaces

## Basic Operations

### Opening Apps

There are several ways to open applications:

1. **App Launcher**: Click "Activities" and browse apps
2. **Dock**: Click an app icon in the dock
3. **Search**: Press Super and type the app name
4. **Command**: Press Alt+F2 and type the command

### Managing Windows

- **Move**: Drag the title bar
- **Resize**: Drag the edges or corners
- **Maximize**: Double-click the title bar or drag to the top
- **Minimize**: Click the "-" button
- **Close**: Click the "×" button or press Alt+F4
- **Switch**: Alt+Tab or click the window in the overview

### Notifications

Notifications appear in the top-right:

```
┌──────────────────────────────┐
│ 🔔 Firefox            ⛶   │
│ Download completed           │
│                        [Open] │
└──────────────────────────────┘
```

- Click to expand and see details
- Click "Close" to dismiss
- Click action buttons to respond
- Check history by clicking "⛶"

## Customization

### Changing Wallpaper

1. Right-click on the desktop
2. Select "Change Background"
3. Choose from available wallpapers
4. Or click "Add Wallpaper" to use your own

### Adjusting Brightness

- Click the brightness icon in the panel
- Drag the slider to adjust
- On mobile, use brightness buttons

### Changing Theme

1. Open Settings
2. Go to "Appearance"
3. Choose between:
   - Light
   - Dark
   - Auto (switches based on time of day)

### Adding App Shortcuts

1. Open the app launcher
2. Find the app you want
3. Right-click the app icon
4. Select "Add to Dock"

## Getting Help

- **System Settings**: Access all settings from the panel
- **Help App**: Built-in help and tutorials
- **Online**: Visit https://help.rustux.com
- **Community**: Join our forums at https://community.rustux.com

## Tips and Tricks

### Keyboard Shortcuts

- **Super**: Open app launcher
- **Super+Tab**: Switch windows
- **Super+Arrow Keys**: Move windows
- **Super+Enter**: Open terminal
- **Alt+F1**: Open menu
- **Alt+F2**: Command dialog
- **Ctrl+Alt+Arrow Keys**: Switch workspaces
- **Print Screen**: Take screenshot

### Touch Gestures (Tablet/Mobile)

- **Swipe from left edge**: Open app drawer
- **Swipe from right edge**: Quick settings
- **Swipe up**: Go to home screen
- **Pinch**: Zoom in/out
- **Two-finger swipe**: Switch workspace

### Split View

1. Open two apps
2. Drag one window's title bar to the left edge
3. Drop when the edge highlights
4. Adjust the divider as needed

## Next Steps

- Explore the built-in applications
- Customize your desktop
- Install new apps from the software store
- Set up your online accounts
- Explore the accessibility features

For more detailed guides, see the other sections in this documentation.
```

## Application Guides

```markdown
# File Manager (Files)

## Overview

The Files app allows you to browse, manage, and organize your files and folders.

![Files App](../images/files-app.png)

## Basic Operations

### Navigating

- **Sidebar**: Quick access to common locations
- **Breadcrumb Bar**: See current location and navigate up
- **Back/Forward**: Use the toolbar buttons

### Creating Folders

1. Click the "New Folder" button in the toolbar
2. Type the folder name
3. Press Enter

### Moving/Copying Files

**Drag and Drop:**
- Drag files between folders
- Hold Ctrl to copy instead of move

**Context Menu:**
1. Right-click the file
2. Select "Cut" or "Copy"
3. Navigate to destination
4. Right-click and select "Paste"

### Deleting Files

1. Select the file(s)
2. Press Delete or right-click → "Move to Trash"
3. Files stay in trash until you empty it

### Renaming

1. Right-click the file
2. Select "Rename"
3. Type the new name
4. Press Enter

### Properties

View file properties:
1. Right-click the file
2. Select "Properties"

Shows:
- Name
- Type
- Size
- Created/Modified dates
- Permissions

## Advanced Features

### Search

Click the search icon and type to search:
- **Current Folder**: Search only in current location
- **All Files**: Search everywhere

### Sorting

Click column headers to sort:
- Name
- Date
- Size
- Type

### View Modes

Switch between views using the view buttons:
- **Grid**: Icon grid view
- **List**: Detailed list view
- **Columns**: Multi-column view

### Compressed Files

Work with archives:
- **Extract**: Right-click → "Extract Here"
- **Compress**: Select files → Right-click → "Compress"
- **Browse**: Double-click to browse archives

### Network Locations

Access network shares:
1. Click "Other Locations"
2. Enter server address
3. Enter credentials
4. Browse like local files

## Tips

- **Middle Click** on a folder opens it in a new window
- **Ctrl+L** focuses the location bar for quick typing
- **Ctrl+H** toggles hidden files
- **Ctrl+F** starts a search in current folder

## Keyboard Shortcuts

- **Ctrl+N**: New folder
- **Ctrl+Shift+N**: New window
- **Ctrl+R**: Reload
- **Ctrl+H**: Show hidden
- **Ctrl+F**: Search
- **Ctrl+A**: Select all
- **Delete**: Move to trash
- **Shift+Delete**: Delete immediately
```

## Customization Guide

```markdown
# Customizing Your Desktop

## Appearance

### Theme

Choose your preferred theme:

1. Open Settings
2. Go to "Appearance"
3. Select from:
   - **Light**: Bright, light-colored theme
   - **Dark**: Dark theme for low-light environments
   - **Auto**: Automatically switches between light and dark

### Accent Color

Personalize with an accent color:

1. Open Settings
2. Go to "Appearance"
3. Choose from preset colors or pick a custom one

Accent color affects:
- Buttons
- Links
- Progress bars
- Selection highlights

### Fonts

Adjust fonts:

1. Open Settings
2. Go to "Appearance"
3. Click "Fonts"
4. Choose:
   - Interface font
   - Document font
   - Monospace font
   - Font sizes
   - Antialiasing

### Icons

Change icon theme:

1. Open Settings
3. Choose from available icon themes

### Cursor

Customize the cursor:

1. Open Settings
2. Go to "Appearance"
3. Click "Cursor"
4. Choose:
   - Cursor size
   - Cursor theme

## Desktop

### Wallpaper

Set your wallpaper:

1. Right-click on desktop
2. Select "Change Background"
3. Choose:
   - **Wallpapers**: Built-in wallpapers
   - **Colors**: Solid color background
   - **Picture**: Use your own image

**Slideshow**: Set up rotating wallpapers:
1. Open Settings
2. Go to "Background"
3. Click "Slideshow"
4. Select folder and interval

### Dock

Customize the dock:

**Position:**
1. Right-click the dock
2. Select "Dock Settings"
3. Choose position: Left, Right, or Bottom

**Size:**
- Drag the handle to resize
- Or in Dock Settings, use the size slider

**Auto-hide:**
1. Right-click the dock
2. Select "Dock Settings"
3. Enable "Auto-hide"

**Icon Size:**
1. Right-click the dock
2. Select "Dock Settings"
3. Adjust the icon size slider

### Panel

Configure the panel:

**Position:** Not yet customizable (future feature)

**Panel Contents:**
1. Open Settings
2. Go to "Panel"
3. Toggle items on/off:
   - Clock
   - Calendar
   - System Tray
   - Status icons

## Behavior

### Windows

Configure window behavior:

1. Open Settings
2. Go to "Windows"
3. Options:
   - **Focus**: Click to focus or focus follows mouse
   - **New Windows**: Centered or cascaded
   - **Title Bar Actions**: Configure title bar buttons

### Workspaces

Set workspace preferences:

1. Open Settings
2. Go to "Workspaces"
3. Options:
   - **Dynamic Workspaces**: Auto-add/remove workspaces
   - **Workspace Switcher**: Enable/disable workspace indicator
   - **Number of Workspaces**: Fixed number of workspaces

### Notifications

Control notifications:

1. Open Settings
2. Go to "Notifications"
3. Configure:
   - **Lock Screen Notifications**: Show/hide on lock screen
   - **Notification Popups**: Position and duration

## Extensions

### Installing Extensions

Add new functionality:

1. Open the Application Library
2. Browse to "Extensions"
3. Find an extension
4. Click "Install"

### Managing Extensions

Manage installed extensions:

1. Open Settings
2. Go to "Extensions"
3. View:
   - Enabled extensions
   - Extension settings
   - Remove extension

## Accessibility

### High Contrast

Enable high contrast mode:

1. Open Settings
2. Go to "Accessibility"
3. Enable "High Contrast"

### Large Text

Increase text size:

1. Open Settings
2. Go to "Accessibility"
3. Adjust "Large Text" slider

### Screen Reader

Enable screen reader:

1. Open Settings
2. Go to "Accessibility"
3. Enable "Screen Reader"

## Keyboard Shortcuts

### Customizing Shortcuts

Set your own shortcuts:

1. Open Settings
2. Go to "Keyboard"
3. Click "Customize Shortcuts"
4. Find the action
5. Click the current shortcut
6. Press your desired key combination

### Common Shortcuts

- **Super**: Open app launcher
- **Super+Tab**: Switch windows
- **Super+A**: Show applications
- **Super+D**: Show desktop
- **Super+L**: Lock screen
- **Print Screen**: Screenshot
- **Alt+F1**: Application menu
- **Alt+F2**: Run command
- **Alt+Tab**: Switch windows
- **Ctrl+Alt+Arrow Keys**: Switch workspace
```

## Mobile Guide

```markdown
# Using Rustica OS on Mobile Devices

## Introduction

Rustica OS is designed to work seamlessly on tablets and mobile devices. This guide covers mobile-specific features.

## Touch Basics

### Navigation Gestures

**Home Screen:**
- **Swipe Up**: Go to home screen from anywhere
- **Swipe Up and Hold**: Show recent apps

**Quick Settings:**
- **Swipe Down from Top**: Open quick settings
- **Swipe Down Again**: Expand quick settings

**Notifications:**
- **Swipe Down from Top**: View notifications
- **Swipe Up**: Hide notification shade

**Back Navigation:**
- **Swipe from Left Edge**: Go back
- **Swipe from Right Edge**: Forward (in apps that support it)

### On-Screen Keyboard

**Typing:**
1. Tap a text field to open the keyboard
2. Tap keys to type
3. Tap "?123" for numbers and symbols

**Special Keys:**
- **Shift**: Tap once for next letter, twice for caps lock
- **?123**: Numbers and symbols
- **Emoji**: 😊 to access emoji
- **Globe**: 🌐 to switch languages

**Gesture Typing:**
- Slide your finger from letter to letter
- Lift to enter the word

**Keyboard Settings:**
1. Go to Settings → Keyboard
2. Options:
   - **Vibrate on keypress**
   - **Sound on keypress**
   - **Auto-correct**
   - **Show suggestions**
   - **Keyboard layouts**

## Screen Rotation

### Auto-Rotate

Enable automatic rotation:

1. Swipe down from top for quick settings
2. Tap the rotation icon
3. When enabled, screen rotates when you turn the device

### Manual Rotation

Lock orientation:
1. Swipe down from top for quick settings
2. Tap the rotation lock icon

## Battery Optimization

### Battery Modes

Choose a power profile:

1. Swipe down from top for quick settings
2. Tap the battery icon
3. Choose:
   - **Performance**: Max performance
   - **Balanced**: Balanced performance
   - **Power Saver**: Extend battery life
   - **Battery Saver**: Maximum battery life

### Battery Saver

Enable battery saver:
- Automatically enabled at 20% battery
- Reduces performance
- Limits background activity
- Lowers screen brightness

### Battery Usage

Check battery usage:

1. Go to Settings
2. Go to "Battery"
3. See which apps are using the most power

## Multitasking

### Split Screen

Run two apps side by side:

1. Open the first app
2. Swipe up and hold to see recent apps
3. Drag the first app to the left or right edge
4. Select the second app from recent apps
5. Adjust the divider

### Picture-in-Picture

Watch videos while using other apps:

1. Start playing a video
2. Go to home screen (video minimizes to corner)
3. Drag the PiP window to reposition it
4. Tap to expand, X to close

## Notifications on Mobile

### Managing Notifications

**View Notifications:**
- Swipe down from top
- Scroll through notifications
- Tap to open the app

**Quick Actions:**
- Swipe left on notification for actions
- Swipe right to dismiss

**Notification Settings:**
1. Long-press the notification
2. Tap "⚙" or tap "More"
3. Configure notification behavior for that app

## Apps on Mobile

### Optimizing Apps

Some apps have mobile-specific features:

**Tablet Mode:**
- Apps detect tablet form factor
- UI adapts accordingly
- More screen space for content

**Touch Gestures in Apps:**
- **Pinch**: Zoom in/out
- **Rotate**: Rotate content
- **Two-finger swipe**: Back/forward
- **Long-press**: Context menu

### Full Screen Apps

Use app in full screen:
- Tap the app
- System UI hides for full immersion
- Swipe from edge to reveal system UI

## Privacy & Security

### Lock Screen

Secure your device:

1. Go to Settings → Security
2. Set up:
   - **PIN**: Quick numeric PIN
   - **Password**: Secure password
   - **Fingerprint**: Biometric unlock (if supported)

### Smart Lock

Options for convenient security:

1. Go to Settings → Security
2. Smart Lock options:
   - **On-body detection**: Stay unlocked while on you
   - **Trusted places**: Unlock at home/work
   - **Trusted devices**: Unlock when near your phone/watch

### App Permissions

Control app access:

1. Go to Settings → Apps
2. Select an app
3. Go to "Permissions"
4. Toggle permissions:
   - Camera
   - Microphone
   - Location
   - Contacts
   - Storage
   - Notifications

## Tips

### Taking Screenshots

- **Power + Volume Down**: Take screenshot
- **Power + Volume Down (hold)**: Take partial screenshot

### Force Restart

If device freezes:
1. Hold Power + Volume Down
2. Hold for 10 seconds
3. Device restarts

### Bootloader Mode

Enter bootloader:
1. Power off device
2. Hold Volume Down
3. Connect USB cable
4. Continue holding until bootloader screen
```

## Troubleshooting

```markdown
# Troubleshooting

## Common Issues

### Black Screen After Boot

**Possible Causes:**
- Graphics driver issue
- Display configuration problem
- Kernel panic

**Solutions:**

1. **Check Display Connection**
   - Ensure cable is securely connected
   - Try a different cable or port

2. **Boot to Safe Mode**
   - Hold Shift during boot
   - Select "Rustica OS (Safe Mode)"

3. **Reconfigure Display**
   ```bash
   # Run display configuration
   rustica-display-config --auto
   ```

4. **Update Graphics Driver**
   ```bash
   sudo apt update
   sudo apt install mesa-utils
   ```

### Apps Not Starting

**Possible Causes:**
- Missing dependencies
- Corrupted installation
- Permission issues

**Solutions:**

1. **Check Logs**
   ```bash
   journalctl -xe
   ```

2. **Reinstall App**
   ```bash
   rustica-pm reinstall org.example.App
   ```

3. **Reset App Settings**
   ```bash
   rm -rf ~/.config/org.example.App
   ```

4. **Check Dependencies**
   ```bash
   rustica-pm check-deps org.example.App
   ```

### WiFi Not Working

**Possible Causes:**
- Driver not loaded
- Firmware missing
- Configuration issue

**Solutions:**

1. **Check WiFi Adapter**
   ```bash
   nmcli device status
   ```

2. **Enable WiFi**
   ```bash
   nmcli radio wifi on
   ```

3. **Scan Networks**
   ```bash
   nmcli device wifi list
   ```

4. **Connect**
   ```bash
   nmcli device wifi connect "SSID" password "password"
   ```

### Touch Screen Not Responding

**Possible Causes:**
- Driver issue
- Calibration needed
- Hardware problem

**Solutions:**

1. **Recalibrate**
   ```bash
   rustica-touch-calibrate
   ```

2. **Check Driver**
   ```bash
   dmesg | grep -i touch
   ```

3. **Restart Touch Service**
   ```bash
   systemctl restart --user rustica-touch
   ```

### System Running Slowly

**Possible Causes:**
- Too many background apps
- Insufficient RAM
- High CPU usage

**Solutions:**

1. **Check Resource Usage**
   ```bash
   htop
   ```

2. **Close Background Apps**
   - Open Activity Monitor
   - Quit unused apps

3. **Enable Battery Saver**
   - Switch to power saving mode

4. **Reduce Animations**
   - Go to Settings → Accessibility
   - Enable "Reduce Animation"

### Sound Not Working

**Possible Causes:**
- Audio device not selected
- Volume muted
- Audio service not running

**Solutions:**

1. **Check Audio Service**
   ```bash
   systemctl --user status pulseaudio
   ```

2. **Restart Audio Service**
   ```bash
   systemctl --user restart pulseaudio
   ```

3. **Check Volume**
   - Click volume icon in panel
   - Ensure not muted

4. **Select Correct Output**
   - Go to Settings → Sound
   - Choose output device

### Battery Draining Quickly

**Possible Causes:**
- Battery hungry apps
- Brightness too high
- Power saving disabled

**Solutions:**

1. **Check Battery Usage**
   - Go to Settings → Battery
   - See which apps use the most power

2. **Enable Power Saver**
   - Go to Settings → Battery
   - Enable "Battery Saver"

3. **Reduce Brightness**
   - Lower screen brightness

4. **Close Unused Apps**
   - Quit apps running in background

### Unable to Install Apps

**Possible Causes:**
- Storage full
- No internet connection
- Package manager issue

**Solutions:**

1. **Check Storage**
   ```bash
   df -h
   ```

2. **Clean Package Cache**
   ```bash
   rustica-pm clean
   ```

3. **Check Connection**
   ```bash
   ping -c 3 api.rustux.com
   ```

## Hardware Issues

### WiFi Not Detected

1. Check if adapter is recognized:
   ```bash
   lspci | grep -i network
   ```

2. Install drivers if needed:
   ```bash
   sudo apt install firmware-iwlwifi
   ```

3. Enable device:
   ```bash
   sudo rfkill unblock wifi
   ```

### Bluetooth Not Working

1. Check Bluetooth status:
   ```bash
   bluetoothctl
   ```

2. Enable Bluetooth:
   ```bash
   sudo rfkill unblock bluetooth
   ```

3. Restart service:
   ```bash
   systemctl restart bluetooth
   ```

### External Monitor Not Detected

1. Check connection:
   ```bash
   xrandr
   ```

2. Force detection:
   ```bash
   xrandr --auto
   ```

3. Configure manually:
   ```bash
   xrandr --output HDMI-1 --auto
   ```

## Recovery

### System Recovery

If system won't boot:

1. **Boot to Recovery Mode**
   - Hold Shift during boot
   - Select "Advanced Options"
   - Select "Rustica OS (Recovery Mode)"

2. **Repair Installation**
   ```bash
   rustica-repair
   ```

3. **Reinstall Kernel**
   ```bash
   rustica-kernel-install
   ```

### Reset to Factory Settings

**Warning:** This erases all data

1. Boot from installation media
2. Select "Reinstall Rustica OS"
3. Choose "Erase disk and reinstall"
4. Follow setup wizard

### Backup Your Data

**Automated Backup:**
1. Go to Settings → System → Backups
2. Add backup location
3. Enable automatic backups

**Manual Backup:**
```bash
# Backup home directory
rsync -av ~/ /backup/location/

# Backup installed packages
rustica-pm list > ~/packages.txt
rustica-pm backup-packages ~/packages.txt
```

## Getting More Help

### System Logs

Collect logs for bug reports:

```bash
# Current boot logs
journalctl -b

• Previous boot logs
journalctl -b -1

• All boot logs
journalctl --list-boots

• Save logs to file
journalctl -b > ~/boot-logs.txt
```

### Bug Reports

Report bugs:
1. Go to https://github.com/rustux/rustica-gui/issues
2. Search for existing issues
3. Create new issue with:
   - Description
   - Steps to reproduce
   - Expected behavior
   - Actual behavior
   - System information
   - Logs

### Community Support

Get help from the community:
- **Forum**: https://community.rustux.com
- **Matrix**: #rustica:matrix.org
- **Email**: support@rustux.com
- **IRC**: #rustica on Libera.Chat
```

## Configuration

```toml
# User documentation build configuration
[book]
title = "Rustica OS User Guide"
authors = ["Rustica OS Team"]
src = "docs/user"
language = "en"

[build]
build-dir = "docs/user/book"
create-missing = false

[preprocessor.toc]
command = "mdbook-toc"
renderer = ["html"]

[preprocessor.katex]
command = "mdbook-katex"
renderers = ["html"]

[output.html]
default-theme = "rust"
preferred-dark-theme = "rust"
git-repository-url = "https://github.com/rustux/rustica-gui"
edit-url-template = "https://github.com/rustux/rustica-gui/edit/main/docs/user/{path}"

[output.html.search]
enable = true
limit-results = 30

[output.print]
enable = true
page-break = true
```

## Best Practices

1. **User-Centric**: Write from user perspective
2. **Simple Language**: Avoid technical jargon
3. **Screenshots**: Include relevant screenshots
4. **Step-by-Step**: Break down complex tasks
5. **Context**: Explain why, not just how
6. **Scenarios**: Cover real-world use cases
7. **Troubleshooting**: Include common issues
8. **Searchable**: Use clear headings and keywords
9. **Translations**: Prepare for localization
10. **Regular Updates**: Keep docs updated with releases

## Tools

- **mdBook**: Book format documentation
- **ScreenShooter**: Screenshot capture tool
- **Kazam**: Screencast recording
- **Shutter**: Screenshot annotation
- **asciinema**: Terminal recording
- **Pandoc**: Format conversion

## Future Enhancements

1. **Interactive Tutorials**: Step-by-step interactive guides
2. **Video Tutorials**: Embedded video demonstrations
3. **Context Help**: F1 help from anywhere in the UI
4. **Tooltips**: Detailed tooltips for all UI elements
5. **Guided Tours**: First-run guided tours
6. **Cheat Sheets**: Quick reference cards
7. **FAQ**: Frequently Asked Questions
8. **User Forums**: Community-driven support
9. **Knowledge Base**: Searchable knowledge base
10. **Mobile App**: Documentation mobile app
