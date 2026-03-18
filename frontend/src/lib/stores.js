import { writable, derived } from 'svelte/store';

export const authToken = writable(localStorage.getItem('agentboard_token') || null);
export const teams = writable({});
export const pendingPlan = writable(null);
export const connected = writable(false);
export const currentView = writable(localStorage.getItem('agentboard_token') ? 'home' : 'pair');
export const selectedTeamId = writable(null);

authToken.subscribe(val => {
    if (val) localStorage.setItem('agentboard_token', val);
    else localStorage.removeItem('agentboard_token');
});

export const teamsList = derived(teams, $teams => Object.values($teams));
export const completedCount = derived(teamsList, $list => $list.filter(t => t.status === 'Done').length);
export const totalCount = derived(teamsList, $list => $list.length);
