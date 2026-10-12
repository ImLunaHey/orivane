# Forms

The Forms module collects structured responses through native Discord modals.

- Members use `/form name` to open an enabled form.
- Each form has one to five short or paragraph questions with Discord-enforced length limits.
- Forms can be available to everyone or restricted to selected roles.
- Completed responses are routed to a configured text channel. User-authored mentions are neutralized so submissions cannot ping members or roles.
- Anonymous forms hide the submitter in the routed message and do not retain a submitter ID alongside the answers. The protected audit event still identifies the actor for abuse investigations.
- Completed submissions and their answers are stored in PostgreSQL and can be reviewed from the form's dashboard inbox. Each submission has a dedicated detail page.
- Audit events contain the submission ID and outcome, never submitted answers.

Form configuration and submissions live in PostgreSQL. Access to submission list and detail endpoints uses the same server authorization checks as form configuration, and every read is audited without copying response content into the audit record. The module has its own Rust crate at `modules/forms`, and its Discord boundary is fully mockable through `bot-test-support`.
