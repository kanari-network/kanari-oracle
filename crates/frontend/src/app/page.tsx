import Link from "next/link";
import { Button } from "@/components/ui/Button";
import { Card, CardBody } from "@/components/ui/Card";

export default function Home() {
  return (
    <div className="min-h-[calc(100vh-4rem)]">
      {/* Hero Section */}
      <section className="relative overflow-hidden">
        <div className="absolute inset-0 bg-gradient-to-br from-purple-500/10 via-blue-500/10 to-pink-500/10"></div>
        <div className="relative max-w-7xl mx-auto px-4 sm:px-6 lg:px-8 py-24 sm:py-32">
          <div className="text-center">
            <h1 className="text-5xl sm:text-6xl lg:text-7xl font-bold text-gray-900 dark:text-white mb-6">
              <span className="bg-gradient-to-r from-purple-600 via-blue-600 to-pink-600 bg-clip-text text-transparent">
                Kanari Oracle
              </span>
            </h1>
            <p className="text-xl sm:text-2xl text-gray-600 dark:text-gray-300 mb-8 max-w-3xl mx-auto">
              Real-time cryptocurrency and stock price data for your web3 applications, 
              trading bots, and financial services
            </p>
            <div className="flex gap-4 justify-center flex-wrap">
              <Link href="/register">
                <Button size="lg">
                  Get Started Free
                </Button>
              </Link>
              <Link href="/dashboard">
                <Button variant="secondary" size="lg">
                  View Dashboard
                </Button>
              </Link>
            </div>
          </div>
        </div>
      </section>

      {/* Features Section */}
      <section className="py-20 bg-white/50 dark:bg-gray-900/50">
        <div className="max-w-7xl mx-auto px-4 sm:px-6 lg:px-8">
          <h2 className="text-3xl sm:text-4xl font-bold text-center text-gray-900 dark:text-white mb-12">
            Powerful Features
          </h2>
          <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-3 gap-8">
            <Card hover>
              <CardBody className="text-center">
                <div className="text-5xl mb-4">⚡</div>
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                  Real-time Data
                </h3>
                <p className="text-gray-600 dark:text-gray-400">
                  Get live price updates every 30 seconds with multiple data source fallbacks
                </p>
              </CardBody>
            </Card>

            <Card hover>
              <CardBody className="text-center">
                <div className="text-5xl mb-4">🔐</div>
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                  Secure API
                </h3>
                <p className="text-gray-600 dark:text-gray-400">
                  Token-based authentication with Argon2id password hashing
                </p>
              </CardBody>
            </Card>

            <Card hover>
              <CardBody className="text-center">
                <div className="text-5xl mb-4">📊</div>
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                  Multi-Asset Support
                </h3>
                <p className="text-gray-600 dark:text-gray-400">
                  Access both cryptocurrency and stock market data in one place
                </p>
              </CardBody>
            </Card>

            <Card hover>
              <CardBody className="text-center">
                <div className="text-5xl mb-4">🚀</div>
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                  High Performance
                </h3>
                <p className="text-gray-600 dark:text-gray-400">
                  Built with Rust for maximum speed and reliability
                </p>
              </CardBody>
            </Card>

            <Card hover>
              <CardBody className="text-center">
                <div className="text-5xl mb-4">🔄</div>
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                  Auto Updates
                </h3>
                <p className="text-gray-600 dark:text-gray-400">
                  Background price updates ensure you always have fresh data
                </p>
              </CardBody>
            </Card>

            <Card hover>
              <CardBody className="text-center">
                <div className="text-5xl mb-4">💼</div>
                <h3 className="text-xl font-semibold text-gray-900 dark:text-white mb-2">
                  Developer Friendly
                </h3>
                <p className="text-gray-600 dark:text-gray-400">
                  RESTful API with comprehensive documentation and SDKs
                </p>
              </CardBody>
            </Card>
          </div>
        </div>
      </section>

      {/* CTA Section */}
      <section className="py-20">
        <div className="max-w-4xl mx-auto px-4 sm:px-6 lg:px-8 text-center">
          <Card className="bg-gradient-to-r from-purple-600 to-blue-600">
            <CardBody className="py-12">
              <h2 className="text-3xl sm:text-4xl font-bold text-white mb-4">
                Ready to Get Started?
              </h2>
              <p className="text-xl text-purple-100 mb-8">
                Create your free account and start accessing real-time price data today
              </p>
              <Link href="/register">
                <Button variant="secondary" size="lg">
                  Create Free Account
                </Button>
              </Link>
            </CardBody>
          </Card>
        </div>
      </section>
    </div>

  );
}
