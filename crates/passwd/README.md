# zroj_core/passwd

This crate implements the password crypto.

When registering a new user, apply [`register_hash`] to the plain password text (in the front end).

When login to an existing user, apply [`login_hash`] to the plain password text (in the front end).

In the backend, use [`verify`] to check the validity of user input password.