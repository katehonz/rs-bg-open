import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import { AuthProvider } from './context/AuthContext';
import ProtectedRoute from './components/ProtectedRoute';
import Layout from './components/layout/Layout';
import Login from './pages/Login';
import Dashboard from './pages/Dashboard';
import Settings from './pages/Settings';
import Currencies from './pages/Currencies';
import ChartOfAccounts from './pages/ChartOfAccounts';
import JournalEntry from './pages/JournalEntry';
import VATEntry from './pages/VATEntry';
import VAT from './pages/VAT';
import VATRates from './pages/VATRates';
import ImportCenter from './pages/ImportCenter';
import SafTExport from './pages/SafTExport';
import NotFound from './pages/NotFound';
import FixedAssets from './pages/FixedAssets';
import Inventory from './pages/Inventory';
import JournalEntriesList from './pages/JournalEntriesList';
import Reports from './pages/Reports';
import Intrastat from './pages/Intrastat';
import Counterparts from './pages/Counterparts';
import ErrorBoundary from './components/ErrorBoundary';
import Banks from './pages/Banks';
import MonthlyStats from './pages/MonthlyStats';
import CounterpartyReports from './pages/CounterpartyReports';

function App() {
  return (
    <AuthProvider>
      <Router>
        <Routes>
          {/* Public Routes */}
          <Route path="/login" element={<Login />} />

          {/* Protected Routes with Layout */}
          <Route
            path="/"
            element={
              <ProtectedRoute>
                <Layout />
              </ProtectedRoute>
            }
          >
            <Route index element={<Navigate to="/dashboard" replace />} />
            <Route path="dashboard" element={<Dashboard />} />
            <Route path="settings/*" element={<Settings />} />

            {/* Placeholder routes for other pages */}
            <Route path="accounting/entries" element={<JournalEntry />} />
            <Route path="accounting/journal-entries" element={<JournalEntriesList />} />
            <Route path="accounting/vat-entry" element={<VATEntry />} />
            <Route path="accounting/chart" element={<ChartOfAccounts />} />
            <Route path="vat/returns" element={<VAT />} />
            <Route path="vat/rates" element={<VATRates />} />
            <Route path="imports" element={<ImportCenter />} />
            <Route path="banks" element={<Banks />} />
            <Route path="reports" element={<Reports />} />
            <Route path="reports/monthly-stats" element={<MonthlyStats />} />
            <Route path="reports/counterparty-turnover" element={<CounterpartyReports />} />
            <Route path="saft-export" element={<SafTExport />} />
            <Route path="currencies" element={<Currencies />} />
            <Route path="counterparts" element={<ErrorBoundary><Counterparts /></ErrorBoundary>} />
            <Route path="fixed-assets" element={<ErrorBoundary><FixedAssets /></ErrorBoundary>} />
            <Route path="inventory" element={<ErrorBoundary><Inventory /></ErrorBoundary>} />
            <Route path="intrastat/*" element={<ErrorBoundary><Intrastat /></ErrorBoundary>} />
          </Route>

          {/* 404 Route */}
          <Route path="*" element={<NotFound />} />
        </Routes>
      </Router>
    </AuthProvider>
  );
}

export default App;
