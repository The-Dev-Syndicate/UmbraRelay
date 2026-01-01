# Troubleshooting

Common issues and solutions for UmbraRelay.

## Sources Not Syncing

### RSS Feed Issues

**Problem**: RSS feed not updating

**Solutions**:
1. **Check URL**: Verify the RSS URL is correct and accessible
2. **Test in Browser**: Open the URL in a browser to verify it works
3. **Manual Sync**: Try clicking "Sync" button to test
4. **Check Poll Interval**: Ensure poll interval is reasonable (not too short)
5. **Feed Format**: Verify the feed is valid RSS/Atom

**Error Messages**:
- "Failed to fetch RSS feed": Network issue or invalid URL
- "Failed to parse RSS feed": Feed format issue

### GitHub Issues

**Problem**: GitHub source not syncing

**Solutions**:
1. **Check Token**: Verify GitHub token is valid and not expired
2. **Token Scopes**: Ensure token has `repo` or `public_repo` scope
3. **Repository Access**: Verify you have access to the repository
4. **Rate Limits**: Check if you've hit GitHub API rate limits
5. **Manual Sync**: Try clicking "Sync" button

**Error Messages**:
- "GitHub token not found": Token not stored properly
- "GitHub API error: 401": Invalid or expired token
- "GitHub API error: 403": Rate limit or permission issue
- "GitHub API error: 404": Repository not found

## Database Issues

**Problem**: Database errors or corruption

**Solutions**:
1. **Restart App**: Close and reopen UmbraRelay
2. **Check Permissions**: Ensure app has write permissions
3. **Database Location**: Check database file exists and is accessible
4. **Backup**: Backup database before troubleshooting

**Database Location**:
- App data directory (platform-specific)
- File: `umbrarelay.db`

## Configuration Issues

**Problem**: Sources not saving or configuration errors

**Solutions**:
1. **Check Database**: Ensure database is accessible and writable
2. **Permissions**: Ensure app has write permissions to data directory
3. **Restart App**: Try restarting the app if changes aren't saving
4. **Re-add Source**: If a source isn't working, try removing and re-adding it

## GitHub Authentication Problems

**Problem**: Can't authenticate with GitHub

**Solutions**:
1. **Token Generation**: Generate a new token
2. **Token Scopes**: Ensure correct scopes are selected
3. **Token Storage**: Re-add source to store token
4. **Token Expiration**: Check if token has expired

**Token Requirements**:
- Must be a "classic" personal access token
- Requires `repo` scope for private repos
- Requires `public_repo` scope for public repos only

## Performance Issues

**Problem**: App is slow or unresponsive

**Solutions**:
1. **Too Many Sources**: Reduce number of sources
2. **Poll Intervals**: Increase poll intervals
3. **Large Database**: Archive old items
4. **System Resources**: Check system resources

## Items Not Appearing

**Problem**: Items not showing in inbox

**Solutions**:
1. **Check Filters**: Verify you're not filtering items out
2. **Source Enabled**: Ensure source is enabled
3. **Sync Source**: Manually sync the source
4. **Check State**: Items might be archived

## App Won't Start

**Problem**: UmbraRelay won't launch

**Solutions**:
1. **Check Logs**: Check system logs for errors
2. **Permissions**: Ensure app has necessary permissions
3. **Dependencies**: Verify all dependencies are installed
4. **Reinstall**: Try reinstalling the app

## Reporting Bugs

If you encounter a bug:

1. **Check Logs**: Look for error messages
2. **Reproduce**: Try to reproduce the issue
3. **Document**: Note steps to reproduce
4. **Report**: Open an issue on GitHub with:
   - Description of the problem
   - Steps to reproduce
   - Error messages (if any)
   - System information (OS, version)

## Getting Help

- **Documentation**: Check other documentation sections
- **Examples**: See [Use Cases](../examples/use-cases.md)
- **GitHub Issues**: Search existing issues or open a new one
- **Development Guide**: See [Development](../technical/development.md) for advanced troubleshooting

