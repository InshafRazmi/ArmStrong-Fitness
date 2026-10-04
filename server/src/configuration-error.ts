// Only application-authored configuration messages may be shown in startup
// logs. Keep environment values, credentials and file paths out of this type.
export class ServerConfigurationError extends Error {}
