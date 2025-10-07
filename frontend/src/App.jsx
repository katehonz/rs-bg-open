import { BrowserRouter as Router, Routes, Route, Navigate } from 'react-router-dom';
import Layout from './components/layout/Layout';
import Dashboard from './pages/Dashboard';
import Settings from './pages/Settings';
import Currencies from './pages/Currencies';
import ChartOfAccounts from './pages/ChartOfAccounts';
import JournalEntry from './pages/JournalEntry';
import VATEntry from './pages/VATEntry';
import VAT from './pages/VAT';
import ImportCenter from './pages/ImportCenter';
import SafTExport from './pages/SafTExport';
import NotFound from './pages/NotFound';
import FixedAssets from './pages/FixedAssets';
import JournalEntriesList from './pages/JournalEntriesList';
import Reports from './pages/Reports';
import Intrastat from './pages/Intrastat';
import Counterparts from './pages/Counterparts';
import ErrorBoundary from './components/ErrorBoundary';
import Banks from './pages/Banks';

function App() {
  return (
    <Router>
      <Routes>
        {/* Routes with Layout */}
        <Route path="/" element={<Layout />}>
          <Route index element={<Navigate to="/dashboard" replace />} />
          <Route path="dashboard" element={<Dashboard />} />
          <Route path="settings/*" element={<Settings />} />
          
          {/* Placeholder routes for other pages */}
          <Route path="accounting/entries" element={<JournalEntry />} />
          <Route path="accounting/journal-entries" element={<JournalEntriesList />} />
          <Route path="accounting/vat-entry" element={<VATEntry />} />
          <Route path="accounting/chart" element={<ChartOfAccounts />} />
          <Route path="vat/*" element={<VAT />} />
          <Route path="imports" element={<ImportCenter />} />
          <Route path="banks" element={<Banks />} />
          <Route path="reports" element={<Reports />} />
          <Route path="saft-export" element={<SafTExport />} />
          <Route path="currencies" element={<Currencies />} />
          <Route path="counterparts" element={<ErrorBoundary><Counterparts /></ErrorBoundary>} />
          <Route path="fixed-assets" element={<ErrorBoundary><FixedAssets /></ErrorBoundary>} />
          <Route path="intrastat/*" element={<ErrorBoundary><Intrastat /></ErrorBoundary>} />
        </Route>
        
        {/* 404 Route */}
        <Route path="*" element={<NotFound />} />
      </Routes>
    </Router>
  );
}

export default App;
