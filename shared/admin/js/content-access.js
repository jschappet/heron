// Check the session before requesting capabilities that are hidden from visitors.
// Read the profile directly: the page layout may already be loading the auth store.
async function requireContentLogin() {
  let profile;
  try {
    profile = await Alpine.store('api').call('profile_json');
  } catch (error) {
    // Older backends omit the profile capability for anonymous visitors.
    if (error.status !== 401 && error.message !== 'Unknown API function: profile_json') throw error;
  }
  if (!profile?.authenticated) {
    window.location.replace('/login/');
    return false;
  }
  return true;
}
