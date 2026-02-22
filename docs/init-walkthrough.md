# devflow init Walkthrough

1. Run `devflow init` in the repository root.
2. Edit `.devflow.yaml` and define:
   - required env keys and expected types
   - services and startup commands
   - test command
   - ignore globs
   - desired ports
3. Add `.env` and run `devflow up`.
4. Use `devflow env doctor` to verify local machine state.
