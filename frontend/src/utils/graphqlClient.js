// Simple GraphQL client for RS-AC-BG
// This is a basic implementation without Apollo Client

const GRAPHQL_ENDPOINT = '/graphql';

export async function graphqlRequest(query, variables = {}) {
  try {
    const token = localStorage.getItem('authToken');
    const headers = {
      'Content-Type': 'application/json',
    };

    if (token) {
      headers['Authorization'] = `Bearer ${token}`;
    }

    const response = await fetch(GRAPHQL_ENDPOINT, {
      method: 'POST',
      headers,
      body: JSON.stringify({
        query,
        variables,
      }),
    });

    if (!response.ok) {
      throw new Error(`HTTP error! status: ${response.status}`);
    }

    const result = await response.json();
    
    if (result.errors) {
      throw new Error(result.errors.map(err => err.message).join(', '));
    }

    return result.data;
  } catch (error) {
    console.error('GraphQL request failed:', error);
    throw error;
  }
}

// Helper function for mutations
export async function graphqlMutation(mutation, variables = {}) {
  return graphqlRequest(mutation, variables);
}

// Helper function for queries
export async function graphqlQuery(query, variables = {}) {
  return graphqlRequest(query, variables);
}

export default {
  request: graphqlRequest,
  mutation: graphqlMutation,
  query: graphqlQuery,
};