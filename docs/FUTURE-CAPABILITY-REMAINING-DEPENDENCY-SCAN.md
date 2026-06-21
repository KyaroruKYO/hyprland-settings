# Future Capability Remaining Dependency Scan

## 2026-06-20

- Completed a dependency scan across the remaining tracker categories after adding draft-edit review plumbing, the default-disabled persistence boundary, and the default-disabled production activation safety gates.
- Core UI/navigation, config discovery/source-aware modeling, 341-row model, safe normal-scalar writes, and release packaging are effectively capped for the current safe-release scope.
- Missing/default insertion and duplicate resolution are blocked by production activation because their review/form/draft/persistence-boundary/safety-gate layers exist but executors remain intentionally unwired and production-critical proof remains missing.
- Structured hl.bind writes, profile/mode switching, and runtime/reload integration are also blocked by production activation.
- High-risk/display recovery remains blocked by high-risk recovery proof requirements.
- Hyprland 0.55.4 migration remains blocked by missing trusted official export data, row-count diff, write-safety review, and safe-env evidence.
- The safe independent follow-up areas completed were the default-disabled activation draft persistence boundary and the default-disabled production activation safety gates. Remaining implementation now requires production activation safety proof, high-risk recovery proof, official 0.55.4 export data, or a user release decision.
- No production behavior, persistence, real config mutation, runtime mutation, reload, or migration activation was added.
