export function updateCheckError(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error)
  if (/could not fetch a valid release json/i.test(message)) {
    return 'The Windows update feed is unavailable or invalid. Check your connection and ask the Administrator to verify the published signed release. You can install an updated setup file manually; saved gym records are retained.'
  }
  return message
}
