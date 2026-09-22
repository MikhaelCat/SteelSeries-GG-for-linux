# Issue Templates

## Bug Report
---
name: 🐛 Bug Report
about: Report a reproducible bug
title: '[BUG] '
labels: ['bug', 'triage']

---

**Describe the bug**
A clear and concise description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Connect device: [e.g., Apex Pro TKL]
2. Command: [e.g., 'ssgg rgb color -c red']
3. Expected: [Expected behavior]
4. Actual: [Actual behavior]

**Expected behavior**
A clear and concise description of what you expected to happen.

**System Information:**
- OS: [e.g., Ubuntu 22.04, Fedora 38]
- Kernel: $(uname -r)
- ssgg version: $(ssgg --version)
- Rust version: $(rustc --version)

**Additional context**
Add any other context about the problem here.

## Feature Request
---
name: ✨ Feature Request
about: Suggest an enhancement
title: '[FEATURE] '
labels: ['enhancement', 'triage']

---

**Problem Statement**
What problem would this feature solve?

**Proposed Solution**
How should this feature work?

**Alternatives Considered**
What other approaches were considered?

**Additional Context**
Any relevant examples or mockups?

---

## Pull Request Template

### Description
[Provide a brief summary of the changes]

### Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update

### Testing
- [ ] Tests pass locally with `cargo test`
- [ ] Clippy warnings resolved (`cargo clippy`)
- [ ] Code formatted (`cargo fmt`)
- [ ] New code covered with unit tests
- [ ] Integration tested with real hardware (if applicable)

### Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex logic
- [ ] Documentation updated (README, CHANGELOG, etc.)
- [ ] No unnecessary whitespace changes
- [ ] Branch rebased on latest master
