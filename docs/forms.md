# Forms

The Forms module collects structured responses through native Discord modals.

- Members use `/form name` to open an enabled form.
- Each form has one to five short or paragraph questions with Discord-enforced length limits.
- Forms can be available to everyone or restricted to selected roles.
- Completed responses are routed to a configured text channel. User-authored mentions are neutralized so submissions cannot ping members or roles.
- Anonymous forms hide the submitter in the routed message. The actor ID remains in the protected audit event for abuse investigations.
- Audit events contain the form ID and outcome, never submitted answers.

Form configuration lives in PostgreSQL with the rest of the guild configuration. The module has its own Rust crate at `modules/forms`, and its Discord boundary is fully mockable through `bot-test-support`.
