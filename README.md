# ![icon](https://raw.githubusercontent.com/MCOfficer/bitwarden-autotype/master/assets/icon.png) bitwarden-autotype
#### The missing desktop-autotype for Bitwarden.


Autotype/Autofill support has been a requested feature in Bitwarden [for years](https://community.bitwarden.com/t/auto-type-autofill-for-logging-into-other-desktop-apps/158), yet no progress seems to have been made. Queue the Thanos GIF!

![thanos](https://media1.tenor.com/images/3f5a7b7a5fc637975f7a962874ace47d/tenor.gif)

## Features

- [x] Log into your Bitwarden Vault
- [x] Use a global hotkey (Ctrl-Alt-A) to trigger autotyping
- [ ] Configurable hotkey
- [x] Match window titles against Bitwarden Login URLs
- [x] Choose between multiple matching Logins
- [x] Autotype following the `{USERNAME}{TAB}{PASSWORD}{ENTER}` Pattern
- [x] Custom Autotype Patterns (see [#1](https://github.com/MCOfficer/bitwarden-autotype/issues/1))
- [ ] Autotype in windows with elevated permissions (see [#5](https://github.com/MCOfficer/bitwarden-autotype/issues/5))
- [x] Ability to sync your Vault automatically and manually
- [ ] Pretty UI

## How To

- Install [Bitwarden CLI](https://bitwarden.com/help/article/cli/)
- In Bitwarden, set up your login's URL to match the window's title. You may use any [match detection](https://bitwarden.com/help/article/uri-match-detection/) for this:
  - **Default match detection, Base domain & Host:** Wouldn't recommend, since they're geared towards URLs rather than window titles.
  - **Starts With:** Self-explanatory.
  - **Regular expression:** The most powerful and versatile option, for example the RegEx `.* - Mozilla Firefox` would match any Firefox windows.
  - **Exact:** What it says on the tin - either the window title matches *perfectly*, or it's out.
  - **Never:** Why?
- Download the latest release
- Run the program
- A tray icon appears to signal the program is running. Right click it for more option & information.
- Log into Bitwarden
- In the window you set up the URL for, hit the Autotype hotkey
- Feel the magic flow through you

### Custom Autotype Patterns

If you want to define your own patterns, you can do so in your Login's Notes. Suppose you wanted to only type the password and hit enter, you'd add this line to your Login's Notes:
````
Autotype: {PASSWORD}{ENTER}
````
<sup>(Note: the space after `Autotype:` is required)</sup>

You can specify any pattern you want. Supported are:
- Any Character (some exotic unicode chars may lead to breakage, please report any bugs) except newlines (use `{ENTER}`)
- `{ENTER}` to simulate the enter/return key
- `{TAB}` to simulate the tab key
- `{USERNAME}` to type your login's username field
- `{PASSWORD}` to type your login's password field
- `{SLEEP=X}` to pause typing for X milliseconds

## Caveats

This is a third party program, not affiliated in any way with Bitwarden. It does not store your logins in any way, but it **could**. If you don't trust me and cannot read this code, don't use it.

While this program doesn't store your master password, it has to store a session token so you don't have to enter your password every time you want to Autotype. A malicious program on your PC might be able to steal this token from this program's memory. This may also be true for the official Bitwarden client, though. I'm not a security researcher.

Autotyping has inherent risks. For example, if you hit a hotkey while in your favourite chat program, and have URLs set up to match that program's window title, this program will happily broadcast your username & password to the entire chatroom.

**This is free software, offered without warranty or liability.** I've done my best to create a program I myself can use daily, but don't sue me if it eats your kitten!

## Acknowledgements

Like almost all End-user software, this program rides upon the tip of massive open-source iceberg; thousands and thousands of hours of work which I cannot claim credit for. 

Special mentions include (but are not limited to!) [David Tolnay](https://github.com/dtolnay/), the [fltk](https://www.fltk.org/)- and [fltk-rs](https://github.com/fltk-rs/fltk-rs) Contributors, and everyone working on [The Rust Programming Language](https://www.rust-lang.org/).

Thank you all for the ecosystem you helped create.
