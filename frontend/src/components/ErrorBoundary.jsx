import React from 'react';

class ErrorBoundary extends React.Component {
  constructor(props) {
    super(props);
    this.state = { hasError: false, error: null, errorInfo: null };
  }

  static getDerivedStateFromError() {
    // Update state so the next render will show the fallback UI
    return { hasError: true };
  }

  componentDidCatch(error, errorInfo) {
    // Log the error to console
    console.error('Error Boundary caught an error:', error, errorInfo);
    this.setState({
      error: error,
      errorInfo: errorInfo
    });
  }

  render() {
    if (this.state.hasError) {
      return (
        <div className="p-6">
          <div className="bg-red-50 border border-red-200 rounded-lg p-4">
            <h2 className="text-lg font-semibold text-red-800 mb-4">
              Възникна неочаквана грешка
            </h2>
            <div className="text-sm text-red-700">
              <strong>Грешка:</strong> {this.state.error && this.state.error.toString()}
            </div>
            {this.state.errorInfo && (
              <details className="mt-4 text-sm text-red-600">
                <summary className="cursor-pointer font-medium">Технически детайли</summary>
                <pre className="mt-2 whitespace-pre-wrap bg-red-100 p-2 rounded text-xs">
                  {this.state.errorInfo.componentStack}
                </pre>
              </details>
            )}
            <button 
              onClick={() => window.location.reload()} 
              className="mt-4 px-4 py-2 bg-red-600 text-white rounded hover:bg-red-700"
            >
              Презареди страницата
            </button>
          </div>
        </div>
      );
    }

    return this.props.children;
  }
}

export default ErrorBoundary;
