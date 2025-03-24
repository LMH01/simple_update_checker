# Changelog

## v1.4.0

- number of available updates and programs for which updates are available are now stored in database in update_check_history table
- added subcommand `update-check-history` that can be used to show the history of performed update checks
- fixed display order of table in `update-history`
- fixed `-m` parameter removing the wrong entries `update-history`

## v1.3.0

- notifications for new versions are now only sent once per new version
- notifications are no longer sent in `run-timed` when the update check for the same version was previously performed manually
- added `-a` flag to `check` command:

```
  -a, --allow-notification
          Normally notifications are not sent in run-timed mode for updates that where seen manually.
          Set this flag to not mark the update as seen and to make the notification get sent when run-timed mode is used the next time.
```

- performed updates are now stored in the database, they can be shown using the new `update-history` subcommand

## v1.2.1

- fixed date column showing milliseconds in 'list-programs' command

## v1.2.0

- Programs listed by 'list-programs' are now sorted alphabetically

## v1.1.0

- Last time an update check was performed is now displayed when command 'list-programs' is run
- Current version last updated and latest version last updated times are now saved and are displayed in 'list-programs'

## v1.0.0

- initial release