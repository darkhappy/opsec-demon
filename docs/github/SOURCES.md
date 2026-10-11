# GitHub ornament provenance

Inspected upstream README documentation on 2026-10-11. These are design references and
service providers, not dependencies of the shell. No README prose or artwork was copied.

| Source | Pattern used |
| --- | --- |
| [Shields](https://github.com/badges/shields) | Repository badges: stars, forks, issues, workflow state, license, sizes, and language. |
| [Star History](https://github.com/star-history/star-history) | Calendar and aligned-timeline star charts, multi-repository comparisons, and rank badges. |
| [GitHub Stats Extended](https://github.com/stats-organization/github-stats-extended) | Profile totals, repository pin cards, and language cards; successor to GitHub Readme Stats. |
| [GitHub Readme Stats](https://github.com/anuraghazra/github-readme-stats) | Historical source of the familiar stats-card layout; its README now directs users to the successor. |
| [Streak Stats](https://github.com/DenverCoder1/github-readme-streak-stats) | Account contribution streak cards with custom themes. |
| [Activity Graph](https://github.com/Ashutosh00710/github-readme-activity-graph) | Profile contribution graphs; current documented public deployment uses Vercel. |
| [Profile Trophy](https://github.com/ryo-ma/github-profile-trophy) | Account achievement tiles and themes; hosted service availability varies. |
| [Typing SVG](https://github.com/DenverCoder1/readme-typing-svg) | Animated multiline SVG text for the clearance terminal. |
| [Skill Icons](https://github.com/tandpfun/skill-icons) | Tool logo grids; here they identify configured tool families. |
| [Contribution Snake](https://github.com/Platane/snk) | Generate an animated snake from account contributions using an Action or CLI. |
| [Metrics](https://github.com/lowlighter/metrics) | Generate account infographics with optional plugins; requires setup. |
| [Profile Summary Cards](https://github.com/vn7n24fzkq/github-profile-summary-cards) | Generate profile details, language, productive-time, and commit cards. |
| [Profile Views Counter](https://github.com/antonkomarev/github-profile-views-counter) | Counts image requests for a username, not verified unique human visitors. |
| [Profile README Generator](https://github.com/rahuldkjain/github-profile-readme-generator) | Visual assembly of familiar README add-ons. |
| [Awesome Profile READMEs](https://github.com/abhisheknaiidu/awesome-github-profile-readme) | Gallery for researching additional profile layouts. |

## Popular terminal repositories inspected

- [fish](https://github.com/fish-shell/fish-shell): shell README structure and links.
- [Starship](https://github.com/starship/starship): centered identity, badges, and screenshots.
- [fzf](https://github.com/junegunn/fzf): demonstrations and navigable usage sections.
- [bat](https://github.com/sharkdp/bat): visual examples and project metadata.
- [ripgrep](https://github.com/BurntSushi/ripgrep): capability descriptions with supporting evidence.
- [Neovim](https://github.com/neovim/neovim): project status and contributor navigation.

Current endpoint references: [Shields workflow badge](https://shields.io/badges/git-hub-actions-workflow-status),
[Star History embedding](https://www.star-history.com/blog/how-to-use-github-star-history/),
[Activity Graph deployment](https://github.com/Ashutosh00710/github-readme-activity-graph),
[Stats Extended migration](https://github.com/stats-organization/github-stats-extended#migration-from-github-readme-stats).

The legacy Star History `/svg` endpoint returned SVG HTTP 200 during inspection. Its newer
`/chart` embedding flow may require a generated sealed token. No tokens were copied from
upstream snippets, and none are supplied in these public URLs.
