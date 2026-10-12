# Tray Tasks

**오늘의 할 일을 작업표시줄에서. · Your daily tasks, right on the taskbar.**

**Version 1.0.1 · Windows x64**

[한국어](#한국어) · [English](#english)

![작업표시줄 위젯 / Taskbar widget](docs/assets/widget.jpg)
![할 일 목록 / Task list](docs/assets/tray-tasks.jpg)
![리마인더 / Reminder](docs/assets/reminder.jpg)

## 한국어

Tray Tasks는 옵시디언 데일리 노트의 체크박스를 작업표시줄에서 관리하는 Windows 앱입니다. 오늘의 할 일 하나를 투명 위젯에 표시하고, 아이콘을 누르면 전체 목록이 열립니다. 할 일을 추가하거나 완료하면 데일리 노트의 마크다운에도 반영됩니다.

볼트 폴더에 직접 접근하는 독립 앱으로, 옵시디언을 닫아둔 상태에서도 사용할 수 있습니다.

### 주요 기능

- **작업표시줄 위젯**: 알림 영역 바로 왼쪽에 미완료 할 일을 표시합니다. 작업표시줄보다 위에 유지되며 긴 제목은 천천히 가로로 스크롤됩니다.
- **할 일 관리**: 추가, 수정, 완료, 완료 취소, 삭제를 제공합니다. 들여쓴 체크박스도 각각 독립된 할 일로 다룹니다.
- **목록 정렬**: 드래그로 표시 순서를 바꾸고 재시작 후에도 유지합니다. 데일리 노트의 원문 순서는 바꾸지 않습니다.
- **미루기**: 내일, 다음 주 같은 요일, 직접 선택한 날짜로 옮깁니다. 원본에는 이력이 남고 해당 할 일의 리마인더는 해제됩니다.
- **리마인더**: 오늘·내일의 시간을 선택하고, 여러 알림을 한 창에서 확인합니다. 표시 시간, 반복 간격, 소리를 설정할 수 있습니다.
- **자정 이월**: 전날 미완료 항목을 오늘로 가져올지 묻습니다. 가져올 때 예약된 리마인더 시각은 그대로 유지합니다.
- **자동 실행과 설정**: Windows 로그인 시 실행, 한국어·영어, 글꼴, 아이콘 등을 설정합니다. 옵시디언의 테마와 강조색을 반영합니다.

### 설치와 초기 설정

설치 파일 이름은 **`Tray Tasks_1.0.1_x64-setup.exe`**입니다. 공개된 다운로드는 [Releases](https://github.com/MEKBANSUKOZINGER/Tray-Tasks-Obsidian/releases)에서 확인하거나, 아래 개발 절차로 직접 빌드할 수 있습니다.

1. Windows x64 설치 파일을 실행하고 Tray Tasks를 엽니다.
2. 옵시디언에서 **설정 → 코어 플러그인 → 데일리 노트**를 켭니다. 날짜 형식은 **`YYYY-MM-DD`**로 설정합니다.
3. Tray Tasks에서 **볼트 폴더**를 선택합니다.
4. **데일리 노트 폴더**를 옵시디언과 동일하게 설정합니다. 비워두면 볼트 최상위 폴더를 사용합니다. 예: 볼트가 `D:\Notes`, 데일리 노트 폴더가 `Daily`라면 `D:\Notes\Daily\2026-10-11.md`를 읽습니다.
5. Daily Notes 설정 확인란을 체크하고 저장합니다.

오늘의 노트가 없으면 빈 UTF-8 마크다운 파일을 생성합니다. 기존 노트는 유지하며, 옵시디언 템플릿은 실행하지 않습니다. 실행 환경에는 **Microsoft Edge WebView2**가 필요합니다.

다른 위치에서 만든 할 일이 목록에 보이지 않는다면 날짜 파일명과 데일리 노트 폴더를 먼저 확인하세요. **읽을 제목**을 설정했다면 그 제목 아래의 체크박스만 읽습니다. **추가할 제목**은 새 항목을 넣을 위치입니다. 두 설정을 비워두면 문서 전체를 읽고 문서 끝에 추가합니다. 읽기 범위를 지정할 때는 추가 위치도 그 안에 맞춰주세요.

### 사용 방법

| 동작 | 사용법 |
| --- | --- |
| 전체 목록 열기·닫기 | 위젯 아이콘을 클릭합니다. 트레이 아이콘으로도 열 수 있습니다. |
| 다음 할 일 보기 | 위젯 오른쪽 화살표를 누릅니다. 완료 처리 없이 항목만 넘깁니다. |
| 완료·완료 취소 | 체크박스를 누릅니다. 완료 목록에서 체크를 해제하면 미완료로 돌아옵니다. |
| 추가 | 목록 끝의 추가 버튼을 누르고 입력합니다. Enter 또는 다른 곳을 눌러 저장합니다. 빈 입력은 생성되지 않습니다. |
| 수정 | 제목을 두 번 클릭하거나 우클릭합니다. 제목에 포커스가 있을 때 F2도 사용할 수 있습니다. |
| 순서 변경 | 행 왼쪽 손잡이를 드래그합니다. |
| 삭제·미루기 | 해당 행의 버튼을 사용합니다. 선택한 체크박스 한 개에만 적용됩니다. |
| 데일리 노트 열기 | 목록 하단의 데일리 노트 버튼으로 옵시디언에서 오늘 노트를 엽니다. |
| 완전히 종료 | 트레이 아이콘을 우클릭하고 **Quit / 완전히 종료**를 선택합니다. |

트레이 아이콘이 보이지 않으면 Windows 알림 영역의 펼치기 메뉴를 확인하세요. 아이콘의 표시 위치는 Windows가 관리합니다.

**Windows 로그인 시 자동 실행**을 켜면 로그인 후 위젯과 트레이 아이콘을 표시합니다. 전체 목록과 옵시디언 창은 자동으로 열지 않습니다. 알림을 받으려면 Tray Tasks 자체는 실행 중이어야 합니다.

### 리마인더와 자정 이월

리마인더 버튼을 누르고 할 일을 선택한 뒤, 오늘·내일과 시·분을 고릅니다. 새 예약은 **현재 시각 이후부터 내일 23:59까지** 설정할 수 있습니다. **적용**을 눌러야 저장되며, 취소하면 임시 설정과 임시 해제가 취소됩니다.

- 알림 창의 **확인**은 표시된 리마인더를 해제합니다. 할 일 자체를 완료하지는 않습니다.
- 확인하지 않은 알림은 설정한 표시 시간이 지나면 숨고, 예약 시각 기준으로 설정한 간격마다 반복됩니다.
- 앱 재시작 시 이미 지난 리마인더가 있으면 지난 알림으로 표시합니다.
- 할 일을 완료·삭제하거나 수동으로 미루면 해당 리마인더를 해제합니다.

실행 중 자정이 지나면 전날 미완료 항목을 오늘로 가져올지 묻습니다.

| 선택 | 데일리 노트와 리마인더 |
| --- | --- |
| 가져오기 | 전날 항목을 완료하고 오늘 노트에 새 미완료 항목을 만듭니다. 예약된 리마인더 시각을 유지합니다. |
| 그대로 두기 | 전날 항목을 유지하고 아직 울리지 않은 리마인더만 해제합니다. |

### 마크다운과 저장 위치

일반 마크다운 체크박스를 읽습니다. 코드 블록과 YAML 속성 영역의 체크박스는 제외합니다.

```md
# 오늘의 기록

- [ ] README 작성하기
- [x] 자료 정리하기
```

항목을 구분하고 제목 수정·이동 후에도 추적하기 위해 원문에 숨겨진 ID를 붙입니다.

```md
- [ ] README 작성하기 <!-- tray-tasks:9f53db32-1c7c-4b39-8992-e7748a04d336 -->
```

이 주석은 옵시디언의 읽기 화면에서는 보이지 않습니다. 수동으로 미룬 원본에는 `[>]` 상태와 대상 날짜 링크가 남습니다.

```md
- [>] README 작성하기 <!-- tray-tasks:9f53db32-1c7c-4b39-8992-e7748a04d336 --> → [[2026-10-12]]
```

| 데이터 | 저장 위치 |
| --- | --- |
| 할 일과 완료·미루기 이력 | 선택한 볼트의 데일리 노트 |
| 설정, 표시 순서, 리마인더, 편집 백업 | 일반적으로 `%LOCALAPPDATA%\com.traytasks.obsidian` |

노트 편집 전 백업을 만들고 외부 변경을 확인합니다. 날짜를 옮기는 작업에는 복구 기록을 사용합니다. 볼트 동기화는 사용자가 사용하는 옵시디언 동기화나 다른 도구에서 관리하며, Tray Tasks는 충돌을 자동으로 병합하지 않습니다. 리마인더와 설정은 이 컴퓨터에 저장됩니다.

### 지원 범위

Windows x64 및 `YYYY-MM-DD.md` 데일리 노트를 지원합니다. macOS·Linux, 템플릿 실행, 앱 자체의 클라우드 동기화는 제공하지 않습니다. 옵시디언 Tasks/Reminder 플러그인이 없어도 사용할 수 있습니다.

자동 숨김 작업표시줄, Explorer 재시작, 다중 모니터·배율 변경, 실제 재부팅 자동 실행과 설치 업그레이드의 검증 범위는 [검증 기록](docs/testing.md)에 정리되어 있습니다. Windows 창 순서 테스트를 포함한 자동 테스트는 **45개**입니다.

### 개발하기

준비 환경: Node.js 22.12 이상, Rust MSVC 1.90 이상, Visual Studio C++ 도구, Windows SDK, WebView2.

```powershell
npm ci
npm run desktop:dev
```

`desktop:*` 스크립트는 Visual Studio의 컴파일러 환경을 찾아 실행합니다.

```powershell
npm test                 # TypeScript 테스트
npm run desktop:test     # Rust 테스트
npm run desktop:check    # Rust 컴파일 검사
npm run format:check     # 프론트엔드 포맷 검사
npm run build            # 프론트엔드 빌드
npm run desktop:build    # Windows 설치 파일 생성
```

설치 파일은 `src-tauri/target/release/bundle/nsis/`에 생성됩니다. `desktop:build` 완료 시 최신 설치 파일을 `artifacts/`에 복사하고 이전 버전은 `artifacts/Legacy/`로 옮깁니다. SHA-256 목록도 갱신하며, 해시가 같은 이전 빌드 복사본은 하나로 정리합니다.

UI만 미리 보려면 `npm run dev`를 실행하고 `http://127.0.0.1:1420/?preview`를 엽니다. 미리보기는 예제 데이터를 사용하며 볼트 파일에 접근하지 않습니다. 독립된 개발 실행에서는 `TRAY_TASKS_DATA_DIR`로 테스트 상태 폴더를 지정할 수 있습니다. 이 환경 변수는 릴리스 빌드에서 무시됩니다.

문제를 발견하면 [Issues](https://github.com/MEKBANSUKOZINGER/Tray-Tasks-Obsidian/issues)에 앱 버전, 재현 순서, 예상 결과와 실제 결과를 남겨주세요.

[구현 구조](docs/architecture.md) · [검증 기록](docs/testing.md) · [변경 기록](CHANGELOG.md)

---

## English

Tray Tasks is a Windows app that puts the checkboxes from your Obsidian daily notes on the taskbar. A transparent widget shows one unfinished task; click its icon to open the full list. Adding or completing a task updates the Markdown in your daily note.

The app works directly with your vault folder, so Obsidian can stay closed.

### Features

- **Taskbar widget**: Show an unfinished task just left of the notification area. The widget stays above the taskbar, and long titles scroll slowly.
- **Task management**: Add, rename, complete, reopen and delete tasks. Indented checkboxes are independent tasks.
- **Display order**: Drag rows to reorder the list. Your order persists across restarts without rearranging the Markdown.
- **Postponement**: Move a task to tomorrow, the same weekday next week, or a date you choose. The original keeps a history entry, and the task's reminder is cancelled.
- **Reminders**: Choose a time today or tomorrow, see grouped alerts, and configure display duration, repeat interval and sound.
- **Midnight rollover**: Choose whether to bring yesterday's unfinished tasks into today, preserving their scheduled reminder times.
- **Startup and preferences**: Configure Windows login startup, Korean or English, fonts and icons. The app follows Obsidian's theme and accent color.

### Installation and first run

The installer filename is **`Tray Tasks_1.0.1_x64-setup.exe`**. Check [Releases](https://github.com/MEKBANSUKOZINGER/Tray-Tasks-Obsidian/releases) for published downloads, or build it locally using the development instructions below.

1. Run the Windows x64 installer and open Tray Tasks.
2. In Obsidian, enable **Settings → Core plugins → Daily notes**. Set the date format to **`YYYY-MM-DD`**.
3. Select your **vault folder** in Tray Tasks.
4. Set the **daily-note folder** to match Obsidian. Leave it blank for the vault root. For example, vault `D:\Notes` and daily folder `Daily` use `D:\Notes\Daily\2026-10-11.md`.
5. Check the Daily Notes setup confirmation and save.

Missing daily notes are created as empty UTF-8 Markdown files. Existing notes are kept, and Obsidian templates are not executed. **Microsoft Edge WebView2** is required at runtime.

If tasks are missing from the list, check the filename and daily-note folder first. **Read under heading** limits reading to one heading's section. **Add under heading** sets the insertion section. Leave both blank to read the whole document and append at the end. When limiting reading, keep the insertion section inside that scope.

### Using the app

| Action | How |
| --- | --- |
| Open or close the list | Click the widget icon. You can also use the tray icon. |
| Show the next task | Click the widget's right arrow. It cycles tasks without completing them. |
| Complete or reopen | Click the checkbox. Uncheck a task in the completed list to reopen it. |
| Add | Click Add at the end of the list and type a title. Enter or clicking elsewhere saves it. Empty input is discarded. |
| Rename | Double-click or right-click the title. F2 also works when the title has focus. |
| Reorder | Drag the handle on the left of a row. |
| Delete or postpone | Use that row's buttons. Only the selected checkbox is affected. |
| Open the daily note | Use the daily-note button in the list footer to open today's note in Obsidian. |
| Quit completely | Right-click the tray icon and choose **Quit / 완전히 종료**. |

If the tray icon is hidden, check the notification area's overflow menu. Windows controls its placement.

Enable **Start with Windows** to show the widget and tray icon after login. The full list and Obsidian are kept closed. Tray Tasks must remain running to deliver reminders.

### Reminders and midnight rollover

Click the reminder button, select a task, and choose Today or Tomorrow with the hour and minute wheels. New reminders can be set **from now through tomorrow at 23:59**. Changes are saved only after **Apply**. Cancel discards draft additions and removals.

- **Acknowledge** dismisses the displayed reminders without completing their tasks.
- Unacknowledged alerts hide after the configured duration and repeat at intervals anchored to the scheduled time.
- Restarting with overdue reminders displays a missed-reminder alert.
- Completing, deleting or manually postponing a task cancels its reminder.

When the running app crosses midnight, it asks whether to bring yesterday's unfinished tasks into today.

| Choice | Notes and reminders |
| --- | --- |
| Bring into today | Complete the original entries and create new unfinished tasks in today's note. Keep the scheduled reminder times. |
| Leave in place | Keep the original entries and cancel only reminders that have never rung. |

### Markdown and local data

The app reads standard Markdown checkboxes, excluding fenced code blocks and YAML frontmatter.

```md
# Daily journal

- [ ] Write the README
- [x] Organize research
```

Tasks receive a hidden identifier so they can be tracked after renaming or moving.

```md
- [ ] Write the README <!-- tray-tasks:9f53db32-1c7c-4b39-8992-e7748a04d336 -->
```

The comment is hidden in Obsidian's reading view. A manually postponed original keeps a `[>]` status and a link to the destination date.

```md
- [>] Write the README <!-- tray-tasks:9f53db32-1c7c-4b39-8992-e7748a04d336 --> → [[2026-10-12]]
```

| Data | Location |
| --- | --- |
| Tasks, completion and postponement history | Daily notes in your selected vault |
| Settings, display order, reminders and edit backups | Usually `%LOCALAPPDATA%\com.traytasks.obsidian` |

Edits use a backup and an external-change check. Moves between dates use a recovery journal. Vault syncing is managed by your existing Obsidian sync service or other tools; Tray Tasks does not automatically merge conflicts. Reminders and settings stay on this computer.

### Supported scope

Windows x64 and `YYYY-MM-DD.md` daily notes are supported. macOS, Linux, template execution and built-in cloud sync are not included. Obsidian Tasks/Reminder plugins are not required.

See [verification notes](docs/testing.md) for coverage of auto-hide taskbars, Explorer restart, multiple monitors and DPI changes, reboot startup and installer upgrades. The automated suite contains **45 tests**, including Windows stacking checks.

### Development

Prerequisites: Node.js 22.12 or newer, Rust MSVC 1.90 or newer, Visual Studio C++ tools, Windows SDK and WebView2.

```powershell
npm ci
npm run desktop:dev
```

The `desktop:*` scripts discover Visual Studio and load its compiler environment.

```powershell
npm test                 # TypeScript tests
npm run desktop:test     # Rust tests
npm run desktop:check    # Rust compile check
npm run format:check     # Frontend formatting
npm run build            # Frontend build
npm run desktop:build    # Windows installer
```

The installer is generated under `src-tauri/target/release/bundle/nsis/`. After `desktop:build`, the latest installer is copied to `artifacts/` and older versions are moved to `artifacts/Legacy/`. The SHA-256 manifest is refreshed and identical older build copies are consolidated.

For a UI-only preview, run `npm run dev` and open `http://127.0.0.1:1420/?preview`. Preview mode uses sample data and never accesses a vault. Use `TRAY_TASKS_DATA_DIR` for an isolated state directory in debug builds. Release builds ignore this environment variable.

To report a problem, open an [issue](https://github.com/MEKBANSUKOZINGER/Tray-Tasks-Obsidian/issues) with the app version, steps to reproduce, expected result and actual result.

[Architecture](docs/architecture.md) · [Verification](docs/testing.md) · [Changelog](CHANGELOG.md)
