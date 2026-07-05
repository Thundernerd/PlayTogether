<script setup lang="ts">
import type { CreateGroupResponse, JoinGroupResponse } from '~/types/api'

const { api } = useApi()
const { setToken } = useMemberToken()
const router = useRouter()

const createName = ref('')
const createDisplayName = ref('')
const joinCode = ref('')
const joinDisplayName = ref('')
const loading = ref(false)
const error = ref<string | null>(null)

async function createGroup() {
  loading.value = true
  error.value = null
  try {
    const res = await api<CreateGroupResponse>('/api/groups', {
      method: 'POST',
      body: {
        name: createName.value,
        display_name: createDisplayName.value,
      },
    })
    setToken(res.token)
    await router.push(`/g/${res.group.invite_code}`)
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to create group'
  } finally {
    loading.value = false
  }
}

async function joinGroup() {
  loading.value = true
  error.value = null
  try {
    const code = joinCode.value.trim().toUpperCase()
    const res = await api<JoinGroupResponse>(`/api/groups/by-invite/${code}/join`, {
      method: 'POST',
      body: { display_name: joinDisplayName.value },
    })
    setToken(res.token)
    await router.push(`/g/${res.group.invite_code}`)
  } catch (e: unknown) {
    error.value = e instanceof Error ? e.message : 'Failed to join group'
  } finally {
    loading.value = false
  }
}
</script>

<template>
  <div class="landing">
    <section class="hero">
      <h1>Pick games to play together</h1>
      <p>
        Create a group, invite friends, sync Steam wishlists, and vote on what to play next.
      </p>
    </section>

    <p v-if="error" class="error">{{ error }}</p>

    <div class="panels">
      <section class="card panel">
        <h2 class="section-title">Create a group</h2>
        <label class="label">Group name</label>
        <input v-model="createName" class="input" placeholder="Friday night crew" />

        <label class="label" style="margin-top: 0.75rem">Your display name</label>
        <input v-model="createDisplayName" class="input" placeholder="Alex" />

        <button
          class="btn btn-primary"
          style="margin-top: 1rem"
          :disabled="loading || !createName || !createDisplayName"
          @click="createGroup"
        >
          Create group
        </button>
      </section>

      <section class="card panel">
        <h2 class="section-title">Join with invite code</h2>
        <label class="label">Invite code</label>
        <input v-model="joinCode" class="input" placeholder="AB12CD34" />

        <label class="label" style="margin-top: 0.75rem">Your display name</label>
        <input v-model="joinDisplayName" class="input" placeholder="Sam" />

        <button
          class="btn btn-primary"
          style="margin-top: 1rem"
          :disabled="loading || !joinCode || !joinDisplayName"
          @click="joinGroup"
        >
          Join group
        </button>
      </section>
    </div>
  </div>
</template>

<style scoped>
.landing {
  display: flex;
  flex-direction: column;
  gap: 1.5rem;
}

.hero h1 {
  margin: 0 0 0.5rem;
  font-size: clamp(1.75rem, 4vw, 2.5rem);
}

.hero p {
  margin: 0;
  color: var(--muted);
  max-width: 52ch;
}

.panels {
  display: grid;
  grid-template-columns: repeat(auto-fit, minmax(280px, 1fr));
  gap: 1rem;
}

.panel {
  display: flex;
  flex-direction: column;
}

.error {
  color: #fca5a5;
  margin: 0;
}
</style>
