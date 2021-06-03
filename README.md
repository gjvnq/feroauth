# Ferrocene
A "common base" for Web Apps.

## Main Responsabilities

  * Authentication
    * Logins (TOTP, U2F, mTLS, PGP)
    * Account Recovery
    * Groups
    * Audit logins
  * Logs & Audit
    * Timestamp logs
  * Messaging (at request of connected apps)
    * Sending emails
    * Sending SMS
  * Configuration
    * Database credentials
    * Storing other config info

## Features for a future version (long time away)

  * Doing permissions for apps. (i.e. doesn't fill the role of something like [OPA - Open Policy Agent](www.openpolicyagent.org))

## Quirks

  * Each user can have multiple usernames. (this allows login via email, SSN, computer username, etc)

## Usage

Default user is `admin` and default password is `admin`.