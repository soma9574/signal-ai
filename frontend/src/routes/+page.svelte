<script lang="ts">
	import { onMount } from 'svelte';
	let phoneNumber = '';
	let verificationCode = '';
	let statusMessage = '';
	let isLoading = false;

	function updatePhoneNumber() {
		isLoading = true;
		statusMessage = 'Updating phone number...';
		// API call to backend to update phone number
		fetch('/api/signal/update-number', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ phoneNumber })
		})
			.then(response => response.json())
			.then(data => {
				statusMessage = data.message || 'Phone number updated successfully.';
				isLoading = false;
			})
			.catch(error => {
				statusMessage = `Error updating phone number: ${error.message}`;
				isLoading = false;
			});
	}

	function submitVerificationCode() {
		isLoading = true;
		statusMessage = 'Submitting verification code...';
		// API call to backend to submit verification code
		fetch('/api/signal/verify', {
			method: 'POST',
			headers: { 'Content-Type': 'application/json' },
			body: JSON.stringify({ phoneNumber, verificationCode })
		})
			.then(response => response.json())
			.then(data => {
				statusMessage = data.message || 'Verification code submitted successfully.';
				isLoading = false;
			})
			.catch(error => {
				statusMessage = `Error submitting verification code: ${error.message}`;
				isLoading = false;
			});
	}

	function checkStatus() {
		isLoading = true;
		statusMessage = 'Checking Signal CLI status...';
		// API call to check Signal CLI status
		fetch('/api/signal/status')
			.then(response => response.json())
			.then(data => {
				statusMessage = data.status || 'Signal CLI status checked.';
				isLoading = false;
			})
			.catch(error => {
				statusMessage = `Error checking status: ${error.message}`;
				isLoading = false;
			});
	}

	onMount(() => {
		checkStatus();
	});
</script>

<div class="container mx-auto p-4 max-w-2xl">
	<h1 class="text-3xl font-bold mb-6 text-center">Signal AI Diagnostics</h1>

	<div class="mb-4">
		<label for="phoneNumber" class="block text-sm font-medium text-gray-700">Phone Number</label>
		<input
			id="phoneNumber"
			type="text"
			bind:value={phoneNumber}
			placeholder="+1234567890"
			class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
		/>
		<button
			on:click={updatePhoneNumber}
			disabled={isLoading}
			class="mt-2 w-full bg-indigo-600 text-white py-2 px-4 rounded-md hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-indigo-500 disabled:bg-gray-400"
		>
			Update Phone Number
		</button>
	</div>

	<div class="mb-4">
		<label for="verificationCode" class="block text-sm font-medium text-gray-700">Verification Code</label>
		<input
			id="verificationCode"
			type="text"
			bind:value={verificationCode}
			placeholder="Enter verification code"
			class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-300 focus:ring focus:ring-indigo-200 focus:ring-opacity-50"
		/>
		<button
			on:click={submitVerificationCode}
			disabled={isLoading}
			class="mt-2 w-full bg-green-600 text-white py-2 px-4 rounded-md hover:bg-green-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-green-500 disabled:bg-gray-400"
		>
			Submit Verification Code
		</button>
	</div>

	<div class="mb-4">
		<button
			on:click={checkStatus}
			disabled={isLoading}
			class="w-full bg-blue-600 text-white py-2 px-4 rounded-md hover:bg-blue-700 focus:outline-none focus:ring-2 focus:ring-offset-2 focus:ring-blue-500 disabled:bg-gray-400"
		>
			Check Signal CLI Status
		</button>
	</div>

	<div class="mt-4 p-4 bg-gray-100 rounded-md">
		<p class="text-gray-700">{statusMessage}</p>
		{#if isLoading}
			<div class="animate-spin rounded-full h-6 w-6 border-b-2 border-gray-900 mx-auto mt-2"></div>
		{/if}
	</div>
</div>

<style>
	@import 'tailwindcss';
</style>
