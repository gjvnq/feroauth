# Ferrocene
A "common base" for Web Apps.

## Main Responsabilities

  * Authentication
    * Logins (TOTP, U2F, mTLS, PGP)
    * Account Recovery
    * Groups
    * Permission policy
    * Audit logins
  * Logs & Audit
    * Timestamp logs
  * Messaging (at request of connected apps)
    * Sending emails
    * Sending SMS
  * Configuration
    * Database credentials
    * Storing other config info

## Quirks

  * Each user can have multiple usernames. (this allows login via email, SSN, computer username, etc)
